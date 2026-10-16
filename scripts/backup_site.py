"""SQLite-consistent site backup and isolated restore verification.

Archive contains personal data. Store outside the webroot in a restricted,
encrypted destination. Secrets are deliberately excluded; preserve them in
the operator's secret manager. Never restore directly over a running database.
"""
from contextlib import closing
import argparse
import hashlib
import json
import sqlite3
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def backup(database: Path, output: Path):
    database, output = database.resolve(strict=True), output.resolve()
    if output.exists(): raise ValueError('Refusing to overwrite an existing archive.')
    if output.is_relative_to(ROOT / 'static') or output.is_relative_to(ROOT / 'fonts'):
        raise ValueError('Backups must never be stored in a served directory.')
    output.parent.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='faszien-backup-') as directory:
        snapshot=Path(directory)/'database.sqlite'
        with closing(sqlite3.connect(database.as_uri()+'?mode=ro',uri=True)) as source, closing(sqlite3.connect(snapshot)) as destination:
            source.backup(destination)
            if destination.execute('PRAGMA integrity_check').fetchone()[0]!='ok': raise ValueError('Snapshot failed integrity check.')
            if destination.execute('PRAGMA foreign_key_check').fetchall(): raise ValueError('Snapshot has broken foreign keys.')
        files=[snapshot]
        files.extend(p for folder in ['src','templates','static','fonts'] for p in (ROOT/folder).rglob('*') if p.is_file())
        files.extend(ROOT/name for name in ['Cargo.toml','Cargo.lock','.env.example','robots.txt','sitemap.xml','Logo.avif','Logo.png','Start.avif','og-hero.jpg','favicon.v1.ico','PRODUCT.md'] if (ROOT/name).is_file())
        manifest={}
        with zipfile.ZipFile(output,'x',zipfile.ZIP_DEFLATED) as archive:
            for file in files:
                name='database.sqlite' if file==snapshot else file.relative_to(ROOT).as_posix()
                data=file.read_bytes();manifest[name]=hashlib.sha256(data).hexdigest();archive.writestr(name,data)
            archive.writestr('manifest.json',json.dumps(manifest,indent=2))
    verify(output)

def verify(archive_path: Path):
    with zipfile.ZipFile(archive_path) as archive:
        manifest=json.loads(archive.read('manifest.json'))
        for name,digest in manifest.items():
            if hashlib.sha256(archive.read(name)).hexdigest()!=digest: raise ValueError('Hash mismatch: '+name)
        # Read the known database member only, never extract arbitrary paths.
        with tempfile.TemporaryDirectory(prefix='faszien-restore-test-') as directory:
            restored=Path(directory)/'restored.sqlite';restored.write_bytes(archive.read('database.sqlite'))
            with closing(sqlite3.connect(restored)) as conn:
                assert conn.execute('PRAGMA integrity_check').fetchone()[0]=='ok'
                assert not conn.execute('PRAGMA foreign_key_check').fetchall()
                counts={table:conn.execute('SELECT COUNT(*) FROM '+table).fetchone()[0] for table in ['customers','appointments','payments','withdrawals','email_outbox']}
    print(json.dumps({'archive_verified':True,'files':len(manifest),'restored_counts':counts}))

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('mode',choices=['backup','verify']);parser.add_argument('--database',type=Path);parser.add_argument('--output',type=Path);parser.add_argument('--archive',type=Path);args=parser.parse_args()
    if args.mode=='backup':
        if not args.database or not args.output:parser.error('--database and --output are required')
        backup(args.database,args.output)
    else:
        if not args.archive:parser.error('--archive is required')
        verify(args.archive)
