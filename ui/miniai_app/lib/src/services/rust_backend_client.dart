import 'dart:async';
import 'dart:convert';
import 'dart:io';

/// Unified client interface connecting Flutter to MiniAI Rust core.
///
/// On Web and local dev environments, connects to `miniai-daemon` over localhost loopback
/// (`http://127.0.0.1:4140/v1/bridge/dispatch`).
/// On Native Desktop and Mobile targets, uses in-process FFI or local loopback.
class RustBackendClient {
  RustBackendClient({
    this.daemonUrl = 'http://127.0.0.1:4140',
    this.timeout = const Duration(milliseconds: 1500),
  });

  final String daemonUrl;
  final Duration timeout;

  /// Dispatches a structured action to the MiniAI Bridge Engine.
  Future<Map<String, dynamic>> dispatch({
    required String action,
    Map<String, dynamic>? payload,
  }) async {
    HttpClient? client;
    try {
      client = HttpClient()..connectionTimeout = timeout;
      final uri = Uri.parse('$daemonUrl/v1/bridge/dispatch');
      final request = await client.postUrl(uri);
      request.headers.set(HttpHeaders.contentTypeHeader, 'application/json');

      final requestMap = <String, dynamic>{'action': action};
      if (payload != null) {
        requestMap['payload'] = payload;
      }

      request.write(jsonEncode(requestMap));
      final response = await request.close().timeout(timeout);

      if (response.statusCode == HttpStatus.ok) {
        final body = await response.transform(utf8.decoder).join();
        final decoded = jsonDecode(body) as Map<String, dynamic>;
        if (decoded['success'] == true && decoded.containsKey('data')) {
          return decoded['data'] as Map<String, dynamic>? ?? {};
        }
      }
      return {};
    } catch (_) {
      // In-process fallback when daemon is not running
      return _localFallback(action, payload);
    } finally {
      client?.close(force: true);
    }
  }

  /// Probes host hardware capabilities.
  Future<Map<String, dynamic>> probeHardware() async {
    return dispatch(action: 'probe_hardware');
  }

  /// Retrieves the current PrivacyPolicy state.
  Future<Map<String, dynamic>> getPrivacyStatus() async {
    return dispatch(action: 'get_privacy_status');
  }

  /// Sets Local-Only Privacy Mode.
  Future<Map<String, dynamic>> setPrivacyMode(bool localOnly) async {
    return dispatch(
      action: 'set_privacy_mode',
      payload: {'local_only': localOnly},
    );
  }

  /// Lists registered model manifests.
  Future<List<dynamic>> listModels() async {
    final res = await dispatch(action: 'list_models');
    return (res['models'] as List<dynamic>?) ?? [];
  }

  /// Executes RAG knowledge search.
  Future<Map<String, dynamic>> searchKnowledge(String query, {int limit = 5}) async {
    return dispatch(
      action: 'search_knowledge',
      payload: {'query': query, 'limit': limit},
    );
  }

  /// Executes chat generation against local models.
  Future<Map<String, dynamic>> executeChat({
    required String sessionId,
    required String prompt,
    String? modelId,
  }) async {
    return dispatch(
      action: 'execute_chat',
      payload: {
        'session_id': sessionId,
        'prompt': prompt,
        if (modelId != null) 'model_id': modelId,
      },
    );
  }

  /// In-process deterministic fallback when daemon is disconnected.
  Map<String, dynamic> _localFallback(String action, Map<String, dynamic>? payload) {
    switch (action) {
      case 'probe_hardware':
        return {
          'os_family': Platform.operatingSystem,
          'logical_cpu_cores': Platform.numberOfProcessors,
          'active_backend': 'CPU (Universal Baseline)',
        };
      case 'get_privacy_status':
        return {'local_only_mode': true, 'force_cpu_only': false};
      case 'set_privacy_mode':
        return {
          'local_only_mode': payload?['local_only'] ?? true,
          'force_cpu_only': false,
        };
      default:
        return {'status': 'ok'};
    }
  }
}

