import 'package:flutter/material.dart';
import 'package:matrix_client/session_store.dart';
import 'package:matrix_client/src/rust/api/matrix.dart';
import 'package:matrix_client/theme.dart';

class LoginScreen extends StatefulWidget {
  const LoginScreen({super.key, required this.onLoggedIn});

  final ValueChanged<MatrixClient> onLoggedIn;

  @override
  State<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends State<LoginScreen> {
  final _homeserverController = TextEditingController();
  final _usernameController = TextEditingController();
  final _passwordController = TextEditingController();

  String? _error;
  bool _busy = false;

  Future<void> _login() async {
    setState(() {
      _busy = true;
      _error = null;
    });
    MatrixClient? client;
    try {
      client = await login(
        homeserverUrl: _homeserverController.text,
        username: _usernameController.text,
        password: _passwordController.text,
        storePath: await sqliteStorePath(),
      );
      final session = await clientSession(client: client);
      if (session != null) await saveSession(session);
      widget.onLoggedIn(client);
      client = null;
    } catch (e) {
      // `client` is non-null here only if `login()` itself succeeded but
      // a later step (reading the session, saving it to disk) threw —
      // without disposing it, the native `MatrixClient` leaks, same bug
      // class as the `RoomTimeline`/`MatrixClient` leaks fixed elsewhere
      // in this app. Set to `null` right after a successful hand-off to
      // `onLoggedIn` above so this can't double-dispose a client the
      // parent now owns.
      client?.dispose();
      if (mounted) setState(() => _error = e.toString());
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: MatrixColors.bgContainer,
      body: Center(
        child: SizedBox(
          width: 360,
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const Text(
                  'Matrix client',
                  style: TextStyle(
                    fontSize: 22,
                    fontWeight: FontWeight.bold,
                    color: MatrixColors.onContainer,
                  ),
                ),
                const SizedBox(height: 24),
                if (_error != null) ...[
                  Text(
                    _error!,
                    style: const TextStyle(color: MatrixColors.criticalMain),
                  ),
                  const SizedBox(height: 12),
                ],
                TextField(
                  controller: _homeserverController,
                  style: const TextStyle(color: MatrixColors.onContainer),
                  decoration: const InputDecoration(
                    labelText: 'Homeserver URL',
                    hintText: 'https://matrix.org',
                  ),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: _usernameController,
                  style: const TextStyle(color: MatrixColors.onContainer),
                  decoration: const InputDecoration(labelText: 'Username'),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: _passwordController,
                  obscureText: true,
                  style: const TextStyle(color: MatrixColors.onContainer),
                  decoration: const InputDecoration(labelText: 'Password'),
                  onSubmitted: (_) => _busy ? null : _login(),
                ),
                const SizedBox(height: 20),
                ElevatedButton(
                  onPressed: _busy ? null : _login,
                  child: const Text('Log in'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
