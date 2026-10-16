"""Local contract/privacy regression checks. Uses only the named disposable DB."""
import argparse
import datetime as dt
import html
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
parser.add_argument('--base-url', required=True)
args = parser.parse_args()
assert args.database.name == 'faszien-marketing-preview.db'
assert urllib.parse.urlparse(args.base_url).hostname in ('localhost', '127.0.0.1')
db = sqlite3.connect(args.database)
class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self,req,fp,code,msg,headers,newurl):return None
opener=urllib.request.build_opener(NoRedirect)

def call(route, data=None, cookie='', form=False, method=None):
    body = None if data is None else (urllib.parse.urlencode(data).encode() if form else json.dumps(data).encode())
    headers = {'Origin': args.base_url, 'Content-Type':'application/x-www-form-urlencoded' if form else 'application/json'}
    if cookie: headers['Cookie'] = cookie
    request = urllib.request.Request(args.base_url+route,data=body,headers=headers,method=method)
    try: response = opener.open(request,timeout=15)
    except urllib.error.HTTPError as error: response = error
    with response: return response.status,response.read().decode(),response.headers

for route in ['/buchungsbedingungen','/barrierefreiheit','/widerruf','/impressum','/datenschutz']:
    status, body, headers = call(route)
    assert status == 200 and '{{' not in body, route
    for link in ['/impressum','/datenschutz','/widerruf']: assert f'href="{link}"' in body
assert call('/missing/nested')[0] == 404
assert 'Zur Startseite' in call('/missing/nested')[1]
assert 'Deutsche Telekom-Tochter' not in call('/datenschutz')[1]
assert 'acht Jahren' in call('/datenschutz')[1]
assert 'auth_token' in call('/datenschutz')[1]
print('PASS: information pages, permanent legal links, true 404 and privacy corrections')

name='Compliance '+uuid.uuid4().hex[:8]
statement={'name':name,'email':'receipt@example.test','contract':'Teilwiderruf Testvertrag <script>alert(1)</script>'}
status, review, _=call('/widerruf/pruefen',statement,form=True)
assert status==200 and 'Widerruf bestätigen' in review and '<script>alert(1)</script>' not in review
identifier=html.unescape(re.search(r'name="request_id" value="([^"]+)"',review)[1])
statement['request_id']=identifier
status, receipt, _=call('/widerruf/bestaetigen',statement,form=True)
assert status==200 and 'Dein Widerruf ist eingegangen' in receipt and identifier in receipt
before=db.execute('SELECT COUNT(*) FROM email_outbox').fetchone()[0]
assert call('/widerruf/bestaetigen',statement,form=True)[0]==200
assert db.execute('SELECT COUNT(*) FROM withdrawals WHERE id=?',(identifier,)).fetchone()[0]==1
assert db.execute('SELECT COUNT(*) FROM email_outbox').fetchone()[0]==before
assert db.execute('SELECT COUNT(*) FROM email_outbox WHERE subject LIKE ? AND sent_at IS NULL',('%Widerruf%',)).fetchone()[0]>=2
print('PASS: no-login two-step withdrawal, XSS escaping, durable receipt, outage queue and idempotency')

today=dt.datetime.now(dt.timezone.utc).date() # 21-day horizon stays safe across time-zone date boundaries
date=today+dt.timedelta(days=21)
for _ in range(21):
    status, body, _=call('/api/appointments/available-slots?date='+date.isoformat())
    slots=json.loads(body).get('slots',[])
    if len(slots)>=1:break
    date+=dt.timedelta(days=1)
else:raise AssertionError('No available test slot')
token=uuid.uuid4().hex+uuid.uuid4().hex
email='contract-'+uuid.uuid4().hex[:8]+'@example.test'
db.execute("INSERT INTO booking_requests(token,date,time,first_name,last_name,email,phone,expires_at) VALUES(?,?,?,'Compliance','Test',?,'0123456',datetime('now','+30 minutes'))",(token,date.isoformat(),slots[0]['time'],email))
db.commit()
assert call('/api/guest-booking/confirm',{'token':token},form=True)[0]==409
assert call('/api/guest-booking/confirm',{'token':token,'expected_total':1,'expected_duration':90},form=True)[0]==409
assert db.execute('SELECT COUNT(*) FROM customers WHERE email=?',(email,)).fetchone()[0]==0
status, body, _=call('/api/guest-booking/confirm',{'token':token,'expected_total':195,'expected_duration':90},form=True)
assert status==200,body
appointment=json.loads(body)['appointment_id']
terms=db.execute('SELECT early_start_requested,agreed_total,terms_text FROM booking_terms WHERE appointment_id=?',(appointment,)).fetchone()
assert terms[0]==0 and terms[1]==195 and 'Muster-Widerrufsformular' in terms[2]
assert 'Buchungsnummer: '+str(appointment) in terms[2]
assert call('/api/guest-booking/confirm',{'token':token,'expected_total':195,'expected_duration':90},form=True)[0]==400
print('PASS: changed/missing quote rolls back, consent evidence and full durable booking receipt')

# Reuse the genuine password-reset API to enable the synthetic guest's account.
reset=uuid.uuid4().hex
db.execute("UPDATE customers SET reset_token=?,reset_token_expires=datetime('now','+1 hour') WHERE email=?",(reset,email));db.commit()
assert call('/api/auth/reset-password/'+reset,{'new_password':'compliance-test-only'})[0]==200
status,_,headers=call('/api/auth/login',{'email':email,'password':'compliance-test-only'})
assert status==200
cookie=headers['Set-Cookie'].split(';',1)[0]
status, export, _=call('/api/customer/export-data',cookie=cookie)
assert status==200 and 'booking_confirmations' in export and 'waitlist' in export and 'credit_packages' in export
assert 'password_hash' not in export and 'reset_token' not in export
assert call('/api/customer/delete-account',cookie=cookie,method='DELETE')[0]==409
assert call('/api/appointments/'+str(appointment)+'/cancel',{},cookie=cookie)[0]==200
payment=db.execute('SELECT id FROM payments WHERE appointment_id=?',(appointment,)).fetchone()[0]
assert call('/api/customer/delete-account',cookie=cookie,method='DELETE')[0]==200
archived=db.execute('SELECT record_json,retain_until FROM retained_invoices WHERE payment_id=?',(payment,)).fetchone()
assert archived and 'password' not in archived[0] and 'therapist_notes' not in archived[0]
assert db.execute('SELECT COUNT(*) FROM customers WHERE email=?',(email,)).fetchone()[0]==0
print('PASS: complete account export, live commitment protection, erasure and isolated accounting retention')

status,_,headers=call('/api/auth/login',{'email':'preview@example.test','password':'preview-local-only'})
assert status==200
admin_cookie=headers['Set-Cookie'].split(';',1)[0]
assert name in call('/admin',cookie=admin_cookie)[1]
assert call('/api/admin/retained-invoices',cookie=admin_cookie)[0]==200
assert call('/api/admin/retained-invoices')[0]!=200
assert call('/admin/widerruf/'+identifier+'/bearbeitet',{},cookie=admin_cookie)[0]==303
assert db.execute('SELECT processed_at FROM withdrawals WHERE id=?',(identifier,)).fetchone()[0]
assert 'Aus dem Konto abmelden?' in call('/logout',cookie=admin_cookie)[1]
assert call('/logout',{},cookie=admin_cookie,form=True)[0]==200
assert 'Dashboard' not in call('/admin',cookie=admin_cookie)[1]
print('PASS: admin-only withdrawal processing/archive and server-side logout revocation')
db.close()
