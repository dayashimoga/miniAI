import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/models/system_contracts.dart';
import 'package:miniai_app/src/screens/consent_inspector_screen.dart';
import 'package:miniai_app/src/screens/shell_screen.dart';

void main() {
  group('Subphase 9.3.1 Cloud Consent Contracts & System Models Tests', () {
    test('ConsentActionContract serialization and wire conversions', () {
      expect(ConsentActionContract.granted.wireValue, 'granted');
      expect(ConsentActionContract.verified.wireValue, 'verified');
      expect(ConsentActionContract.revoked.wireValue, 'revoked');
      expect(ConsentActionContract.expired.wireValue, 'expired');
      expect(ConsentActionContract.rejected.wireValue, 'rejected');
      expect(
        ConsentActionContract.payloadDispatched.wireValue,
        'payload_dispatched',
      );

      expect(
        ConsentActionContract.fromWire('granted'),
        ConsentActionContract.granted,
      );
      expect(
        ConsentActionContract.fromWire('verified'),
        ConsentActionContract.verified,
      );
      expect(
        ConsentActionContract.fromWire('revoked'),
        ConsentActionContract.revoked,
      );
      expect(
        ConsentActionContract.fromWire('expired'),
        ConsentActionContract.expired,
      );
      expect(
        ConsentActionContract.fromWire('rejected'),
        ConsentActionContract.rejected,
      );
      expect(
        ConsentActionContract.fromWire('payload_dispatched'),
        ConsentActionContract.payloadDispatched,
      );
      expect(
        ConsentActionContract.fromWire('unknown'),
        ConsentActionContract.rejected,
      );
    });

    test('ConsentAuditRecordContract roundtrip JSON serialization', () {
      const record = ConsentAuditRecordContract(
        id: 42,
        tokenId: 'tok_test_audit',
        action: ConsentActionContract.granted,
        modality: Modality.imageGeneration,
        targetEndpoint: 'https://cloud.miniai.internal/v1',
        payloadBytes: 512,
        epochSecs: 1700000000,
        detailsJson: '{"worker":"gpu_01"}',
      );

      final json = record.toJson();
      expect(json['id'], 42);
      expect(json['token_id'], 'tok_test_audit');
      expect(json['action'], 'granted');
      expect(json['modality'], 'image_generation');
      expect(json['target_endpoint'], 'https://cloud.miniai.internal/v1');
      expect(json['payload_bytes'], 512);
      expect(json['epoch_secs'], 1700000000);
      expect(json['details_json'], '{"worker":"gpu_01"}');

      final parsed = ConsentAuditRecordContract.fromJson(json);
      expect(parsed.id, 42);
      expect(parsed.tokenId, 'tok_test_audit');
      expect(parsed.action, ConsentActionContract.granted);
      expect(parsed.modality, Modality.imageGeneration);
      expect(parsed.targetEndpoint, 'https://cloud.miniai.internal/v1');
      expect(parsed.payloadBytes, 512);
      expect(parsed.epochSecs, 1700000000);
      expect(parsed.detailsJson, '{"worker":"gpu_01"}');
    });

    test('ManagedConsentTokenContract state methods and copyWith', () {
      const token = CloudConsentToken(
        tokenId: 'tok_m_01',
        modality: Modality.video,
        targetEndpoint: 'https://cloud.miniai.internal/v1/video',
        maxPayloadBytes: 10485760,
        issuedAtEpochSecs: 1000,
        expiresAtEpochSecs: 5000,
        userAcknowledged: true,
      );

      var managed = const ManagedConsentTokenContract(
        token: token,
        status: 'active',
        cumulativePayloadBytesUsed: 0,
      );

      expect(managed.isActive, isTrue);
      expect(managed.isRevoked, isFalse);
      expect(managed.isExpired, isFalse);
      expect(managed.isExhausted, isFalse);

      final json = managed.toJson();
      expect(json['status'], 'active');
      expect(json['cumulative_payload_bytes_used'], 0);

      final parsed = ManagedConsentTokenContract.fromJson(json);
      expect(parsed.token.tokenId, 'tok_m_01');
      expect(parsed.isActive, isTrue);

      managed = managed.copyWith(
        status: 'revoked',
        revocationReason: 'Testing revoke',
      );
      expect(managed.isRevoked, isTrue);
      expect(managed.revocationReason, 'Testing revoke');
    });

    test('ComputeTaskRequest and Response contracts JSON roundtrip', () {
      const req = ComputeTaskRequestContract(
        taskId: 'task_roundtrip',
        tokenId: 'tok_req',
        modality: Modality.music,
        operation: 'synthesize',
        payload: 'chord_progression',
        timeoutMs: 3500,
      );

      final reqJson = req.toJson();
      final parsedReq = ComputeTaskRequestContract.fromJson(reqJson);
      expect(parsedReq.taskId, 'task_roundtrip');
      expect(parsedReq.modality, Modality.music);
      expect(parsedReq.timeoutMs, 3500);

      const resp = ComputeTaskResponseContract(
        taskId: 'task_roundtrip',
        workerId: 'cloud_worker_02',
        status: 'success',
        output: 'audio_wav_data',
        latencyMs: 42,
        executedRemotely: true,
        fallbackReason: null,
      );

      final respJson = resp.toJson();
      final parsedResp = ComputeTaskResponseContract.fromJson(respJson);
      expect(parsedResp.taskId, 'task_roundtrip');
      expect(parsedResp.executedRemotely, isTrue);
      expect(parsedResp.status, 'success');
    });
  });

  group('Subphase 9.3.1 PlatformController Cloud Consent State Machine Tests', () {
    test('Default privacy mode blocks consent and falls back locally', () async {
      final controller = PlatformController();
      expect(controller.localOnlyMode, isTrue);
      expect(controller.managedTokens, isEmpty);
      expect(controller.consentAuditHistory, isEmpty);

      const token = CloudConsentToken(
        tokenId: 'tok_blocked',
        modality: Modality.llm,
        targetEndpoint: 'https://cloud.miniai.internal/v1',
        maxPayloadBytes: 10000,
        issuedAtEpochSecs: 1000,
        expiresAtEpochSecs: 5000,
        userAcknowledged: true,
      );

      // Fails closed because localOnlyMode == true
      final granted = controller.grantCloudConsent(token);
      expect(granted, isFalse);
      expect(controller.activeConsent, isNull);
      expect(controller.consentAuditHistory.length, 1);
      expect(
        controller.consentAuditHistory.first.action,
        ConsentActionContract.rejected,
      );

      // Dispatching task with localOnlyMode active triggers automatic local CPU fallback
      const taskReq = ComputeTaskRequestContract(
        taskId: 'task_fb_priv',
        tokenId: 'tok_blocked',
        modality: Modality.llm,
        operation: 'generate',
        payload: 'private text',
        timeoutMs: 1000,
      );

      final response = await controller.dispatchHybridComputeTask(taskReq);
      expect(response.executedRemotely, isFalse);
      expect(response.status, 'success_fallback');
      expect(response.workerId, 'local_cpu_adapter');
      expect(response.output, 'private text:processed_locally_cpu');
      expect(response.fallbackReason, contains('Local-Only Privacy Mode'));
    });

    test('Valid consent grant, task execution, quota update, and exhaustion', () async {
      final controller = PlatformController();
      controller.setLocalOnlyMode(false); // Enable cloud mode

      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final token = CloudConsentToken(
        tokenId: 'tok_quota_test',
        modality: Modality.vision,
        targetEndpoint: 'https://cloud.miniai.internal/v1/vision',
        maxPayloadBytes: 20, // small limit to test exhaustion
        issuedAtEpochSecs: now,
        expiresAtEpochSecs: now + 3600,
        userAcknowledged: true,
      );

      final granted = controller.grantCloudConsent(token);
      expect(granted, isTrue);
      expect(controller.activeConsent, isNotNull);
      expect(controller.managedTokens.length, 1);
      expect(controller.managedTokens.first.isActive, isTrue);

      // Task 1: 10 bytes -> success remotely
      const task1 = ComputeTaskRequestContract(
        taskId: 'task_q1',
        tokenId: 'tok_quota_test',
        modality: Modality.vision,
        operation: 'ocr',
        payload: '1234567890', // 10 bytes
        timeoutMs: 3000,
      );

      final resp1 = await controller.dispatchHybridComputeTask(task1);
      expect(resp1.executedRemotely, isTrue);
      expect(resp1.workerId, 'cloud_gpu_worker_01');
      expect(controller.managedTokens.first.cumulativePayloadBytesUsed, 10);
      expect(controller.managedTokens.first.isActive, isTrue);

      // Task 2: 10 bytes -> reaches 20 bytes -> exhausted!
      const task2 = ComputeTaskRequestContract(
        taskId: 'task_q2',
        tokenId: 'tok_quota_test',
        modality: Modality.vision,
        operation: 'ocr',
        payload: 'abcdefghij', // 10 bytes
        timeoutMs: 3000,
      );

      final resp2 = await controller.dispatchHybridComputeTask(task2);
      expect(resp2.executedRemotely, isTrue);
      expect(controller.managedTokens.first.cumulativePayloadBytesUsed, 20);
      expect(controller.managedTokens.first.isExhausted, isTrue);

      // Task 3: Token is now exhausted -> triggers local CPU fallback!
      const task3 = ComputeTaskRequestContract(
        taskId: 'task_q3',
        tokenId: 'tok_quota_test',
        modality: Modality.vision,
        operation: 'ocr',
        payload: 'fallback_payload',
        timeoutMs: 3000,
      );

      final resp3 = await controller.dispatchHybridComputeTask(task3);
      expect(resp3.executedRemotely, isFalse);
      expect(resp3.status, 'success_fallback');
      expect(resp3.fallbackReason, contains('exhausted'));
    });

    test('Simulated remote 500 error and timeout trigger local CPU fallback', () async {
      final controller = PlatformController();
      controller.setLocalOnlyMode(false);

      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final token = CloudConsentToken(
        tokenId: 'tok_err_test',
        modality: Modality.speechTts,
        targetEndpoint: 'https://cloud.miniai.internal/v1/tts',
        maxPayloadBytes: 10000,
        issuedAtEpochSecs: now,
        expiresAtEpochSecs: now + 3600,
        userAcknowledged: true,
      );
      controller.grantCloudConsent(token);

      const task = ComputeTaskRequestContract(
        taskId: 'task_err',
        tokenId: 'tok_err_test',
        modality: Modality.speechTts,
        operation: 'tts',
        payload: 'hello speech',
        timeoutMs: 1500,
      );

      // 1. Simulate 500 error
      final resp500 = await controller.dispatchHybridComputeTask(
        task,
        simulateRemoteFailure: true,
      );
      expect(resp500.executedRemotely, isFalse);
      expect(resp500.status, 'success_fallback');
      expect(resp500.fallbackReason, contains('500'));
      expect(resp500.output, 'hello speech:processed_locally_cpu');

      // 2. Simulate timeout
      final respTo = await controller.dispatchHybridComputeTask(
        task,
        simulateRemoteTimeout: true,
      );
      expect(respTo.executedRemotely, isFalse);
      expect(respTo.status, 'success_fallback');
      expect(respTo.fallbackReason, contains('timed out'));
    });

    test('Emergency kill-switch revokes all tokens and enforces local privacy mode', () {
      final controller = PlatformController();
      controller.setLocalOnlyMode(false);

      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final token1 = CloudConsentToken(
        tokenId: 'tok_ks_1',
        modality: Modality.llm,
        targetEndpoint: 'https://cloud.miniai.internal/v1',
        maxPayloadBytes: 1000,
        issuedAtEpochSecs: now,
        expiresAtEpochSecs: now + 3600,
        userAcknowledged: true,
      );
      final token2 = CloudConsentToken(
        tokenId: 'tok_ks_2',
        modality: Modality.vision,
        targetEndpoint: 'https://cloud.miniai.internal/v2',
        maxPayloadBytes: 2000,
        issuedAtEpochSecs: now,
        expiresAtEpochSecs: now + 3600,
        userAcknowledged: true,
      );
      controller.grantCloudConsent(token1);
      controller.grantCloudConsent(token2);
      expect(controller.managedTokens.length, 2);

      controller.emergencyKillSwitch(reason: 'Test kill-switch');
      expect(controller.localOnlyMode, isTrue);
      expect(controller.activeConsent, isNull);
      for (final t in controller.managedTokens) {
        expect(t.isRevoked, isTrue);
      }
    });
  });

  group('Subphase 9.3.1 ConsentInspectorScreen Widget & UI Interaction Tests', () {
    testWidgets('Renders ConsentInspectorScreen with header, tokens, and simulator', (tester) async {
      final controller = PlatformController();

      await tester.pumpWidget(
        MaterialApp(
          home: ConsentInspectorScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('consent_emergency_kill_switch_button')), findsOneWidget);
      expect(find.byKey(const Key('consent_inspector_local_only_switch')), findsOneWidget);
      expect(find.byKey(const Key('open_consent_dialog_button')), findsOneWidget);
      expect(find.byKey(const Key('no_tokens_banner')), findsOneWidget);
      expect(find.byKey(const Key('hybrid_dispatch_simulator_card')), findsOneWidget);
      expect(find.byKey(const Key('consent_audit_ledger_card')), findsOneWidget);
    });

    testWidgets('Opens Grant Consent Dialog, inputs parameters, and grants token', (tester) async {
      final controller = PlatformController();
      controller.setLocalOnlyMode(false); // Enable cloud mode

      await tester.pumpWidget(
        MaterialApp(
          home: ConsentInspectorScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Tap Request Consent button
      await tester.tap(find.byKey(const Key('open_consent_dialog_button')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('consent_grant_dialog')), findsOneWidget);
      expect(find.byKey(const Key('consent_endpoint_input')), findsOneWidget);
      expect(find.byKey(const Key('consent_ack_checkbox')), findsOneWidget);

      // Check user acknowledgement
      await tester.ensureVisible(find.byKey(const Key('consent_ack_checkbox')));
      await tester.tap(find.byKey(const Key('consent_ack_checkbox')));
      await tester.pumpAndSettle();

      // Tap Grant Consent
      await tester.ensureVisible(find.byKey(const Key('grant_consent_submit_button')));
      await tester.tap(find.byKey(const Key('grant_consent_submit_button')));
      await tester.pumpAndSettle();

      // Dialog closed and token appears in list
      expect(find.byKey(const Key('consent_grant_dialog')), findsNothing);
      expect(find.byKey(const Key('managed_tokens_list')), findsOneWidget);
      expect(controller.managedTokens.length, 1);
      expect(controller.managedTokens.first.isActive, isTrue);
    });

    testWidgets('Dispatches hybrid task with local CPU fallback on 500 error in UI', (tester) async {
      final controller = PlatformController();
      controller.setLocalOnlyMode(false);

      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final token = CloudConsentToken(
        tokenId: 'tok_ui_test',
        modality: Modality.llm,
        targetEndpoint: 'https://cloud.miniai.internal/v1',
        maxPayloadBytes: 10000,
        issuedAtEpochSecs: now,
        expiresAtEpochSecs: now + 3600,
        userAcknowledged: true,
      );
      controller.grantCloudConsent(token);

      await tester.pumpWidget(
        MaterialApp(
          home: ConsentInspectorScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Token input
      await tester.ensureVisible(find.byKey(const Key('dispatch_token_id_input')));
      await tester.enterText(
        find.byKey(const Key('dispatch_token_id_input')),
        'tok_ui_test',
      );

      // Check simulate 500 error
      await tester.ensureVisible(find.byKey(const Key('simulate_remote_failure_checkbox')));
      await tester.tap(find.byKey(const Key('simulate_remote_failure_checkbox')));
      await tester.pumpAndSettle();

      // Tap Dispatch Task
      await tester.ensureVisible(find.byKey(const Key('dispatch_compute_task_button')));
      await tester.tap(find.byKey(const Key('dispatch_compute_task_button')));
      await tester.pumpAndSettle();

      // Result card should appear with LOCAL CPU FALLBACK
      expect(find.byKey(const Key('task_result_card')), findsOneWidget);
      expect(find.text('LOCAL CPU FALLBACK'), findsOneWidget);
      expect(find.textContaining('Fallback Reason:'), findsOneWidget);
    });

    testWidgets('Emergency kill-switch button revokes tokens and sets privacy mode', (tester) async {
      final controller = PlatformController();
      controller.setLocalOnlyMode(false);

      final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
      final token = CloudConsentToken(
        tokenId: 'tok_ks_ui',
        modality: Modality.imageGeneration,
        targetEndpoint: 'https://cloud.miniai.internal/v1/image',
        maxPayloadBytes: 10000,
        issuedAtEpochSecs: now,
        expiresAtEpochSecs: now + 3600,
        userAcknowledged: true,
      );
      controller.grantCloudConsent(token);

      await tester.pumpWidget(
        MaterialApp(
          home: ConsentInspectorScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.textContaining('ACTIVE'), findsWidgets);

      // Tap KILL SWITCH
      await tester.tap(find.byKey(const Key('consent_emergency_kill_switch_button')));
      await tester.pumpAndSettle();

      expect(controller.localOnlyMode, isTrue);
      expect(controller.managedTokens.first.isRevoked, isTrue);
      expect(find.text('LOCAL-ONLY PRIVACY (ACTIVE)'), findsOneWidget);
    });

    testWidgets('ShellScreen navigates to Tab 11 Cloud across desktop and mobile', (tester) async {
      final controller = PlatformController();

      // Test Desktop Layout (width = 900)
      tester.view.physicalSize = const Size(900, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);

      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Tap Cloud destination in NavigationRail
      await tester.tap(find.byKey(const Key('nav_rail_cloud')));
      await tester.pumpAndSettle();

      expect(controller.selectedNavigationIndex, 11);
      expect(find.byType(ConsentInspectorScreen), findsOneWidget);

      // Test Mobile Layout (width = 500)
      tester.view.physicalSize = const Size(500, 800);
      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      // Tap privacy status badge in AppBar navigates directly to Cloud Inspector
      controller.setSelectedNavigationIndex(0);
      await tester.pumpAndSettle();
      expect(controller.selectedNavigationIndex, 0);

      await tester.tap(find.byKey(const Key('privacy_status_badge')));
      await tester.pumpAndSettle();
      expect(controller.selectedNavigationIndex, 11);
      expect(find.byType(ConsentInspectorScreen), findsOneWidget);
    });
  });
}
