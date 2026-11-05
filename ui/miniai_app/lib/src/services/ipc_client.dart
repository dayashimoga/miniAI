import 'dart:async';
import 'dart:convert';
import 'dart:io';
import '../models/system_contracts.dart';

/// Client communicating with `crates/miniai-server` over localhost HTTP/SSE loopback.
class IpcClient {
  IpcClient({
    this.baseUrl = 'http://127.0.0.1:4140',
    this.sessionNonce = '',
    this.timeout = const Duration(milliseconds: 500),
  });

  final String baseUrl;
  final String sessionNonce;
  final Duration timeout;

  /// Check server health via unauthenticated `GET /v1/health`.
  Future<bool> checkHealth() async {
    HttpClient? client;
    try {
      client = HttpClient()..connectionTimeout = timeout;
      final request = await client.getUrl(Uri.parse('$baseUrl/v1/health'));
      final response = await request.close();
      return response.statusCode == HttpStatus.ok;
    } catch (_) {
      return false;
    } finally {
      client?.close(force: true);
    }
  }

  /// Fetch full hardware, privacy, and modality snapshot via authenticated `GET /v1/system/snapshot`.
  Future<Map<String, dynamic>?> fetchSystemSnapshot() async {
    HttpClient? client;
    try {
      client = HttpClient()..connectionTimeout = timeout;
      final request = await client.getUrl(Uri.parse('$baseUrl/v1/system/snapshot'));
      request.headers.set(HttpHeaders.authorizationHeader, 'Bearer $sessionNonce');
      final response = await request.close();
      if (response.statusCode == HttpStatus.ok) {
        final body = await response.transform(utf8.decoder).join();
        return jsonDecode(body) as Map<String, dynamic>;
      }
      return null;
    } catch (_) {
      return null;
    } finally {
      client?.close(force: true);
    }
  }

  /// Fetch recent events via authenticated `GET /v1/events/recent`.
  Future<List<EventEnvelope>> fetchRecentEvents({int limit = 50}) async {
    HttpClient? client;
    try {
      client = HttpClient()..connectionTimeout = timeout;
      final uri = Uri.parse('$baseUrl/v1/events/recent?limit=$limit');
      final request = await client.getUrl(uri);
      request.headers.set(HttpHeaders.authorizationHeader, 'Bearer $sessionNonce');
      final response = await request.close();
      if (response.statusCode == HttpStatus.ok) {
        final body = await response.transform(utf8.decoder).join();
        final list = jsonDecode(body) as List<dynamic>;
        return list
            .map((item) => EventEnvelope.fromJson(item as Map<String, dynamic>))
            .toList();
      }
      return [];
    } catch (_) {
      return [];
    } finally {
      client?.close(force: true);
    }
  }

  /// Subscribe to real-time Server-Sent Events via authenticated `GET /v1/events/stream`.
  Stream<EventEnvelope> connectEventStream() async* {
    HttpClient? client;
    try {
      client = HttpClient()..connectionTimeout = timeout;
      final request = await client.getUrl(Uri.parse('$baseUrl/v1/events/stream'));
      request.headers.set(HttpHeaders.authorizationHeader, 'Bearer $sessionNonce');
      request.headers.set(HttpHeaders.acceptHeader, 'text/event-stream');
      final response = await request.close();
      if (response.statusCode != HttpStatus.ok) {
        return;
      }

      final lines = response
          .transform(utf8.decoder)
          .transform(const LineSplitter());

      await for (final line in lines) {
        if (line.startsWith('data: ')) {
          final data = line.substring(6).trim();
          if (data.isNotEmpty) {
            try {
              final json = jsonDecode(data) as Map<String, dynamic>;
              yield EventEnvelope.fromJson(json);
            } catch (_) {
              // Ignore invalid event payload
            }
          }
        }
      }
    } catch (_) {
      // Disconnected or connection closed
    } finally {
      client?.close(force: true);
    }
  }
}
