import 'package:flutter/material.dart';
import 'package:matrix_client/screens/home_screen.dart';
import 'package:matrix_client/screens/login_screen.dart';
import 'package:matrix_client/session_store.dart';
import 'package:matrix_client/src/rust/api/matrix.dart';
import 'package:matrix_client/src/rust/frb_generated.dart';
import 'package:matrix_client/theme.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const MatrixClientApp());
}

enum _Stage { restoring, loggedOut, loggedIn }

class MatrixClientApp extends StatefulWidget {
  const MatrixClientApp({super.key});

  @override
  State<MatrixClientApp> createState() => _MatrixClientAppState();
}

class _MatrixClientAppState extends State<MatrixClientApp> {
  _Stage _stage = _Stage.restoring;
  MatrixClient? _client;

  @override
  void initState() {
    super.initState();
    _tryRestore();
  }

  // A saved session on disk doesn't guarantee it still restores cleanly
  // (revoked/expired token, deleted device) — fall back to the login
  // screen and drop the stale file rather than retrying it forever.
  Future<void> _tryRestore() async {
    final session = await loadSession();
    if (session == null) {
      setState(() => _stage = _Stage.loggedOut);
      return;
    }
    try {
      final client = await restoreSession(
        storePath: await sqliteStorePath(),
        session: session,
      );
      setState(() {
        _client = client;
        _stage = _Stage.loggedIn;
      });
    } catch (_) {
      await clearSession();
      setState(() => _stage = _Stage.loggedOut);
    }
  }

  Future<void> _logout() async {
    final client = _client;
    setState(() {
      _client = null;
      _stage = _Stage.loggedOut;
    });
    if (client != null) {
      await stopSync(client: client);
      // Deterministically drops the underlying Rust `MatrixClient` (and
      // its `matrix-sdk-sqlite` connections) now, instead of whenever
      // Dart's GC gets around to it — `clearSession()` is about to
      // delete the store directory on disk, and on Windows a still-open
      // SQLite handle makes that delete fail with a sharing violation
      // (not an issue on Linux, which allows unlinking open files).
      client.dispose();
    }
    await clearSession();
  }

  @override
  Widget build(BuildContext context) {
    final Widget home;
    switch (_stage) {
      case _Stage.restoring:
        home = const Scaffold(
          backgroundColor: MatrixColors.bgContainer,
          body: Center(
            child: CircularProgressIndicator(color: MatrixColors.primaryMain),
          ),
        );
      case _Stage.loggedOut:
        home = LoginScreen(
          onLoggedIn: (c) => setState(() {
            _client = c;
            _stage = _Stage.loggedIn;
          }),
        );
      case _Stage.loggedIn:
        home = HomeScreen(client: _client!, onLogout: _logout);
    }
    return MaterialApp(theme: buildMatrixTheme(), home: home);
  }
}
