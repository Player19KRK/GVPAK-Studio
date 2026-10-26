"""Exercise remembered browsers against real TLS and real server restarts, using isolated data."""
import hashlib, http.cookiejar, json, os, pathlib, secrets, ssl, subprocess, time, urllib.error, urllib.request

root = pathlib.Path(__file__).resolve().parents[1]
lab = root / 'backend' / ('verification-device-' + secrets.token_hex(4))
lab.mkdir()
team = lab / 'team'
env = {**os.environ, 'GVPAK_PORT': '8795', 'GVPAK_TEAM_PORT': '8796', 'GVPAK_TEAM_HOSTS': 'localhost,127.0.0.1', 'GVPAK_DATA_ROOT': str(lab / 'owner'), 'GVPAK_TEAM_ROOT': str(team), 'GVPAK_PROFILE': 'test', 'GVPAK_FEEDBACK_ROOT': str(lab / 'feedback')}
host, lan = 'http://127.0.0.1:8795', 'https://127.0.0.1:8796'
process = None
checks = []
password = secrets.token_urlsafe(24)

def check(name, ok):
    assert ok, name
    checks.append(name)

def call(base, path, data=None, client=None):
    headers = {'X-GVPAK-IPC': '1'}
    if data is not None:
        headers['Content-Type'] = 'application/json'
    request = urllib.request.Request(base + path, data=None if data is None else json.dumps(data).encode(), headers=headers)
    try:
        with (client or urllib.request).open(request, timeout=20) if client else urllib.request.urlopen(request, timeout=20) as response:
            return response.status, json.loads(response.read()), response.headers
    except urllib.error.HTTPError as error:
        return error.code, json.loads(error.read()), error.headers

def start():
    global process
    process = subprocess.Popen([str(root / 'backend/target/release/gvpak-studio.exe')], cwd=root, env=env, creationflags=getattr(subprocess, 'CREATE_NO_WINDOW', 0), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for _ in range(120):
        if process.poll() is not None:
            raise RuntimeError('Isolated server exited')
        try:
            if call(host, '/api/health')[0] == 200 and (team / 'server.crt').exists():
                return
        except (OSError, ValueError):
            pass
        time.sleep(.25)
    raise RuntimeError('Isolated server startup timed out')

def stop():
    if process is not None and process.poll() is None:
        process.terminate()
        process.wait(timeout=10)

try:
    start()
    context = ssl.create_default_context(cafile=str(team / 'server.crt'))
    def browser():
        return urllib.request.build_opener(urllib.request.ProxyHandler({}), urllib.request.HTTPSHandler(context=context), urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
    check('create isolated account', call(host, '/api/invoke/team_accounts', {'action': 'create', 'username': 'device_qa', 'password': password})[0] == 200)
    first, second, temporary, anonymous = [browser() for _ in range(4)]
    tokens = []
    for client, remember in [(first, True), (second, True), (temporary, False)]:
        status, _, headers = call(lan, '/api/team/login', {'username': 'device_qa', 'password': password, 'remember': remember}, client)
        check('remembered login' if remember else 'temporary login', status == 200)
        cookie = headers['Set-Cookie']
        check('cookie lifetime matches choice', ('Max-Age=2592000' in cookie) if remember else 'Max-Age=' not in cookie)
        check('cookie protections retained', all(flag in cookie for flag in ['HttpOnly', 'Secure', 'SameSite=Strict']))
        tokens.append(cookie.split(';')[0].split('=', 1)[1])
    stored = (team / 'device-sessions.json').read_text()
    check('server persists only token hashes', password not in stored and all(token not in stored for token in tokens) and len(json.loads(stored)) == 2)
    check('anonymous browser stays logged out', call(lan, '/api/team/session', client=anonymous)[1]['user'] is None)
    stop(); start()
    check('first device survives process restart', call(lan, '/api/team/session', client=first)[1]['user'] is not None)
    check('second device survives process restart', call(lan, '/api/team/session', client=second)[1]['user'] is not None)
    check('temporary login does not survive restart', call(lan, '/api/team/session', client=temporary)[1]['user'] is None)
    check('logout succeeds', call(lan, '/api/team/logout', {}, first)[0] == 200)
    stop(); start()
    check('logout remains revoked after restart', call(lan, '/api/team/session', client=first)[1]['user'] is None)
    check('logout leaves second device logged in', call(lan, '/api/team/session', client=second)[1]['user'] is not None)
    check('password reset succeeds', call(host, '/api/invoke/team_accounts', {'action': 'reset', 'username': 'device_qa', 'password': secrets.token_urlsafe(24)})[0] == 200)
    stop(); start()
    check('password reset revokes persisted devices', call(lan, '/api/team/session', client=second)[1]['user'] is None)
    report = {'passed': len(checks), 'checks': checks, 'tlsCertificateVerified': True, 'paidCalls': 0, 'testWorkspace': str(lab)}
    output = root / 'docs/verification/device-login.json'
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
    print(json.dumps({'passed': len(checks), 'report': str(output)}))
finally:
    stop()
