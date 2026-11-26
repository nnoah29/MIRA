import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/unsubscribe.dart';

void expectRefused(List<String> ips) {
  for (final ip in ips) {
    expect(isPublicAddress(InternetAddress(ip)), isFalse, reason: ip);
  }
}

void expectPublic(List<String> ips) {
  for (final ip in ips) {
    expect(isPublicAddress(InternetAddress(ip)), isTrue, reason: ip);
  }
}

void main() {
  test('diffusion limitée 255.255.255.255', () {
    expectRefused(['255.255.255.255']);
  });

  test('réservées 240.0.0.0/4', () {
    expectRefused(['240.0.0.1', '250.1.2.3']);
    expectPublic(['223.255.255.1']);
  });

  test('évaluation 198.18.0.0/15', () {
    expectRefused(['198.18.0.1', '198.19.255.254']);
    expectPublic(['198.17.255.1', '198.20.0.1']);
  });

  test('protocole IETF 192.0.0.0/24', () {
    expectRefused(['192.0.0.8', '192.0.0.170']);
    expectPublic(['192.0.1.1']);
  });

  test('locale de site obsolète fec0::/10', () {
    expectRefused(['fec0::1', 'feff::1']);
  });

  test('IPv4-compatible ::a.b.c.d', () {
    expectRefused(['::808:808', '::a00:1']);
  });

  test('NAT64 64:ff9b::/96 avec IPv4 privée', () {
    expectRefused(['64:ff9b::a00:1', '64:ff9b::7f00:1', '64:ff9b::c0a8:1']);
    expectPublic(['64:ff9b::808:808']);
  });

  test('multicast ff00::/8', () {
    expectRefused(['ff02::1', 'ff0e::1']);
  });
}
