import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/remote/daemon_paths.dart';

void main() {
  test('MIRA_HOME_gives_the_profile_root_under_profiles', () {
    final paths = resolveDaemonPaths(
      profile: 'default',
      env: {'MIRA_HOME': '/m'},
    );
    expect(paths.socket, '/m/profiles/default/daemon.sock');
    expect(paths.token, '/m/profiles/default/daemon.token');
  });

  test('XDG_DATA_HOME_is_used_when_MIRA_HOME_is_absent', () {
    final paths = resolveDaemonPaths(
      profile: 'p',
      env: {'XDG_DATA_HOME': '/x'},
    );
    expect(paths.token, '/x/mira/profiles/p/daemon.token');
  });

  test('HOME_alone_gives_the_local_share_default', () {
    final paths = resolveDaemonPaths(profile: 'p', env: {'HOME': '/h'});
    expect(paths.token, '/h/.local/share/mira/profiles/p/daemon.token');
  });

  test('MIRA_HOME_wins_over_XDG_DATA_HOME_and_HOME_and_empty_values_count_as_absent', () {
    final paths = resolveDaemonPaths(
      profile: 'p',
      env: {'MIRA_HOME': '/m', 'XDG_DATA_HOME': '/x', 'HOME': '/h'},
    );
    expect(paths.token, '/m/profiles/p/daemon.token');
    final empty = resolveDaemonPaths(
      profile: 'p',
      env: {'MIRA_HOME': '', 'XDG_DATA_HOME': '', 'HOME': '/h'},
    );
    expect(empty.token, '/h/.local/share/mira/profiles/p/daemon.token');
  });

  test('an_explicit_home_replaces_MIRA_HOME', () {
    final paths = resolveDaemonPaths(
      profile: 'p',
      env: {'MIRA_HOME': '/m'},
      home: '/e',
    );
    expect(paths.token, '/e/profiles/p/daemon.token');
  });

  test('XDG_RUNTIME_DIR_puts_the_socket_in_mira_daemon_and_falls_back_when_empty_or_absent', () {
    final env = {'MIRA_HOME': '/m', 'XDG_RUNTIME_DIR': '/run/user/1000'};
    expect(
      resolveDaemonPaths(profile: 'p', env: env).socket,
      '/run/user/1000/mira-daemon/p.sock',
    );
    env['XDG_RUNTIME_DIR'] = '';
    expect(
      resolveDaemonPaths(profile: 'p', env: env).socket,
      '/m/profiles/p/daemon.sock',
    );
  });

  test('no_root_or_a_bad_profile_name_raises_ArgumentError', () {
    expect(
      () => resolveDaemonPaths(profile: 'p', env: {}),
      throwsArgumentError,
    );
    for (final bad in ['', '..', '.', 'a/b', r'a\b']) {
      expect(
        () => resolveDaemonPaths(profile: bad, env: {'MIRA_HOME': '/m'}),
        throwsArgumentError,
        reason: bad,
      );
    }
  });
}
