"""Smoke-test the local preview only; never run against a production database."""
import argparse
import datetime
import json
import re
import sqlite3
import urllib.error
import urllib.parse
import urllib.request
import uuid
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--database', type=Path, required=True)
parser.add_argument('--base-url', default='http://127.0.0.1:8088')
args = parser.parse_args()
assert args.database.name == 'faszien-marketing-preview.db', 'Use the explicitly named disposable preview database.'
assert urllib.parse.urlparse(args.base_url).hostname in ('localhost', '127.0.0.1'), 'Local preview only.'
db = sqlite3.connect(args.database)
prefix = 'marketing-smoke-' + uuid.uuid4().hex[:8]

def call(path, data=None, form=False):
    if path == '/api/guest-booking/confirm' and data is not None:
        data = dict(data, expected_total=195, expected_duration=90, early_start_requested="true")
    body = None if data is None else (urllib.parse.urlencode(data).encode() if form else json.dumps(data).encode())
    request = urllib.request.Request(args.base_url + path, data=body, headers={'Origin': args.base_url, 'Content-Type': 'application/x-www-form-urlencoded' if form else 'application/json'})
    try:
        response = urllib.request.urlopen(request, timeout=15)
    except urllib.error.HTTPError as e:
        response = e
    text = response.read().decode()
    return response.status, text, response.headers

for path in ['/', '/buchen', '/faszienbehandlung-ruecken', '/faszienbehandlung-nacken-schulter', '/faszienbehandlung-haltung', '/faszienbehandlung-sport', '/preise-und-ablauf', '/impressum', '/datenschutz']:
    status, html, _ = call(path)
    assert status == 200, (path, status, html[:500])
    assert '{{' not in html and '{%' not in html, path
print('PASS: public pages render without unresolved template syntax')

_, landing, _ = call('/')
for block in re.findall(r'<script type="application/ld\+json">(.*?)</script>', landing, re.S):
    json.loads(block)
assert 'Private Zusatzversicherungen erstatten' not in landing
assert 'schmerzfrei</em>' not in landing
assert '545' not in landing
assert '/buchen' in landing
print('PASS: metadata, reimbursement copy and existing offer')

_, settings, _ = call('/api/settings')
assert not any('nextcloud' in key or 'password' in key or 'secret' in key for key in json.loads(settings))
for path in ['/.env', '/PRODUCT.md', '/src/main.rs', '/Cargo.toml', '/MARKETING.md']:
    assert call(path)[0] == 404, path
print('PASS: no private settings or repository files exposed')

date = datetime.date.today() + datetime.timedelta(days=7)
while date.weekday() != 0:
    date += datetime.timedelta(days=1)
for _ in range(30):
    status, body, _ = call('/api/appointments/available-slots?date=' + str(date))
    slots = json.loads(body)['slots']
    if status == 200 and len(slots) >= 3:
        break
    date += datetime.timedelta(days=1)
else:
    raise AssertionError('No preview day with three free slots was found.')
time = slots[0]['time']

def pending(email, chosen_time=time):
    token = uuid.uuid4().hex + uuid.uuid4().hex
    db.execute("INSERT INTO booking_requests(token,date,time,first_name,last_name,email,phone,is_home_visit,street,zip_code,city,source,expires_at) VALUES(?,?,?,'Test','Kunde',?,'0123456789',0,'','','','Empfehlung',datetime('now','+30 minutes'))", (token,str(date),chosen_time,email))
    db.commit()
    return token

email = prefix + '@example.test'
token = pending(email)
status, _, headers = call('/buchen/bestaetigen/' + token)
assert status == 200 and headers['Referrer-Policy'] == 'no-referrer' and headers['Cache-Control'] == 'no-store'
assert db.execute('SELECT COUNT(*) FROM customers WHERE email=?', (email,)).fetchone()[0] == 0
status, body, _ = call('/api/guest-booking/confirm', {'token':token}, form=True)
assert status == 200 and json.loads(body)['success'], body
appointment = json.loads(body)['appointment_id']
assert db.execute('SELECT COUNT(*) FROM payments WHERE appointment_id=?', (appointment,)).fetchone()[0] == 1
assert db.execute('SELECT source FROM booking_sources WHERE appointment_id=?', (appointment,)).fetchone()[0] == 'Empfehlung'
assert call('/api/guest-booking/confirm', {'token':token}, form=True)[0] == 400
print('PASS: explicit confirmation, atomic appointment/payment/source and one-use token')

# Public guest input must never become executable markup in an admin invoice.
malicious_name = '<img src=x onerror=alert(1)>'
db.execute('UPDATE customers SET first_name=? WHERE email=?',(malicious_name,email))
db.commit()
status, _, login_headers = call('/api/auth/login', {'email':'preview@example.test','password':'preview-local-only'})
assert status == 200
cookie = login_headers.get('Set-Cookie').split(';',1)[0]
payment_id = db.execute('SELECT id FROM payments WHERE appointment_id=?',(appointment,)).fetchone()[0]
request = urllib.request.Request(args.base_url+'/api/customer/invoices/'+str(payment_id),headers={'Cookie':cookie})
invoice = urllib.request.urlopen(request,timeout=10).read().decode()
assert malicious_name not in invoice and '&lt;img src=x onerror=alert(1)&gt;' in invoice
assert '§ 4 Nr. 14' not in invoice and '§ 19 UStG' in invoice
db.execute("UPDATE customers SET first_name='Test' WHERE email=?",(email,))
db.commit()
print('PASS: guest input escaped in invoices; consistent wellness invoice wording')


conflict_email = prefix + '-conflict@example.test'
conflict = pending(conflict_email)
assert call('/api/guest-booking/confirm', {'token':conflict}, form=True)[0] == 409
assert db.execute('SELECT COUNT(*) FROM customers WHERE email=?', (conflict_email,)).fetchone()[0] == 0
print('PASS: conflicting confirmation creates no appointment or guest record')

old_email = prefix + '-existing@example.test'
db.execute("INSERT INTO customers(email,password_hash,first_name,last_name,phone,email_verified) VALUES(?,'unchanged-password','Existing','Profile','original-phone',1)", (old_email,))
old_id = db.execute('SELECT id FROM customers WHERE email=?', (old_email,)).fetchone()[0]
db.execute("INSERT INTO credit_packages(customer_id,total_sessions,used_sessions,price_per_session,valid_until) VALUES(?,10,2,169.90,datetime('now','+1 year'))", (old_id,))
db.commit()
old_token = pending(old_email, slots[1]['time'])
status, body, _ = call('/api/guest-booking/confirm', {'token':old_token}, form=True)
assert status == 200, body
assert db.execute('SELECT password_hash,first_name,last_name,phone FROM customers WHERE id=?',(old_id,)).fetchone() == ('unchanged-password','Existing','Profile','original-phone')
assert db.execute('SELECT used_sessions FROM credit_packages WHERE customer_id=?',(old_id,)).fetchone()[0] == 2
print('PASS: existing profile, credentials and credits stay intact')

expired = pending(prefix + '-expired@example.test', slots[2]['time'])
db.execute("UPDATE booking_requests SET expires_at=datetime('now','-1 minute') WHERE token=?",(expired,))
db.commit()
assert call('/api/guest-booking/confirm', {'token':expired}, form=True)[0] == 400
assert call('/api/appointments/book', {'date':str(date),'time':slots[2]['time']})[0] == 401
assert call('/api/guest-booking/request', {'date':str(date),'time':slots[2]['time'],'first_name':'Test','last_name':'Kunde','phone':'0123','email':'invalid','privacy_acknowledged':True})[0] == 400
print('PASS: expired token, missing login and invalid guest contact rejected')

smtp_email = prefix + '-smtp@example.test'
status, body, _ = call('/api/guest-booking/request', {'date':str(date),'time':slots[2]['time'],'first_name':'Test','last_name':'Kunde','phone':'0123','email':smtp_email,'privacy_acknowledged':True})
assert status == 503 and 'nicht versendet' in json.loads(body)['error'], (status,body)
assert db.execute('SELECT COUNT(*) FROM customers WHERE email=?',(smtp_email,)).fetchone()[0] == 0
print('PASS: SMTP failure is visible and creates no booking')

# Keep a disposable unconfirmed example for browser verification; no external email is sent.
preview_token = pending(prefix + '-preview@example.test', slots[2]['time'])
preview = args.database.parent / 'faszien-marketing-preview-token.txt'
preview.write_text(preview_token, encoding='utf-8')
db.close()
print('All local smoke checks passed. Synthetic preview records remain in the disposable database only.')
