import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/main.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/models/system_contracts.dart';

void main() {
  group('SystemContracts Unit & Serialization Tests', () {
    test('Modality and AcceleratorBackend wire serialization round-trips', () {
      for (final m in Modality.values) {
        expect(Modality.fromWire(m.wireValue), equals(m));
        expect(m.displayName.isNotEmpty, isTrue);
      }
      expect(() => Modality.fromWire('invalid'), throwsArgumentError);

      for (final b in AcceleratorBackend.values) {
        expect(AcceleratorBackend.fromWire(b.wireValue), equals(b));
        expect(b.displayName.isNotEmpty, isTrue);
      }
      expect(AcceleratorBackend.cpu.isCpu, isTrue);
      expect(AcceleratorBackend.cuda.isCpu, isFalse);
      expect(() => AcceleratorBackend.fromWire('invalid'), throwsArgumentError);
    });

    test('HardwareProfile memory admission and CPU fallback logic', () {
      const profile = HardwareProfile(
        osFamily: 'linux',
        arch: 'x86_64',
        logicalCpuCores: 16,
        totalMemoryBytes: 32 * 1024 * 1024 * 1024,
        availableMemoryBytes: 1000,
        detectedBackends: [AcceleratorBackend.cpu, AcceleratorBackend.vulkan],
      );

      expect(profile.safeMemoryLimitBytes, equals(850));
      expect(profile.canAdmitModel(500), isTrue);
      expect(profile.canAdmitModel(0), isFalse);
      expect(profile.canAdmitModel(900), isFalse);

      expect(
        profile.selectExecutionBackend(
          preferred: AcceleratorBackend.cuda,
          forceCpu: false,
        ),
        equals(AcceleratorBackend.vulkan),
      );
      expect(
        profile.selectExecutionBackend(
          preferred: AcceleratorBackend.vulkan,
          forceCpu: false,
        ),
        equals(AcceleratorBackend.vulkan),
      );
      expect(
        profile.selectExecutionBackend(
          preferred: AcceleratorBackend.vulkan,
          forceCpu: true,
        ),
        equals(AcceleratorBackend.cpu),
      );

      final json = profile.toJson();
      final decoded = HardwareProfile.fromJson(json);
      expect(decoded.osFamily, equals('linux'));
      expect(decoded.detectedBackends.length, equals(2));
    });

    test('CloudConsentToken and AdapterHealth JSON round-trip', () {
      const token = CloudConsentToken(
        tokenId: 'tok-01',
        modality: Modality.vision,
        targetEndpoint: 'https://compute.miniai.local',
        maxPayloadBytes: 4096,
        issuedAtEpochSecs: 100,
        expiresAtEpochSecs: 200,
        userAcknowledged: true,
      );
      final tokenDecoded = CloudConsentToken.fromJson(token.toJson());
      expect(tokenDecoded.tokenId, equals('tok-01'));
      expect(
        tokenDecoded.isValidFor(
          requestedModality: Modality.vision,
          endpoint: 'https://compute.miniai.local',
          payloadBytes: 2048,
          currentEpochSecs: 150,
        ),
        isTrue,
      );
      expect(
        tokenDecoded.isValidFor(
          requestedModality: Modality.llm,
          endpoint: 'https://compute.miniai.local',
          payloadBytes: 2048,
          currentEpochSecs: 150,
        ),
        isFalse,
      );

      const health = AdapterHealth(
        adapterId: 'vision-cpu-adapter',
        modality: Modality.vision,
        ready: true,
        activeBackend: AcceleratorBackend.cpu,
        statusMessage: 'Ready',
      );
      final healthDecoded = AdapterHealth.fromJson(health.toJson());
      expect(healthDecoded.adapterId, equals('vision-cpu-adapter'));
      expect(healthDecoded.modality, equals(Modality.vision));
    });

    test('PlatformController blocks invalid consent or Local-Only grant attempts', () {
      final controller = PlatformController();
      expect(controller.localOnlyMode, isTrue);

      const validToken = CloudConsentToken(
        tokenId: 'tok-02',
        modality: Modality.llm,
        targetEndpoint: 'https://compute.miniai.local',
        maxPayloadBytes: 4096,
        issuedAtEpochSecs: 100,
        expiresAtEpochSecs: 200,
        userAcknowledged: true,
      );

      // Cannot grant while Local-Only mode is enabled
      expect(controller.grantCloudConsent(validToken), isFalse);
      expect(
        controller.canExecuteCloudRequest(
          modality: Modality.llm,
          endpoint: 'https://compute.miniai.local',
          payloadBytes: 1024,
          currentEpochSecs: 150,
        ),
        isFalse,
      );

      // Disable Local-Only mode and test invalid token rejection
      controller.setLocalOnlyMode(false);
      const invalidToken = CloudConsentToken(
        tokenId: '',
        modality: Modality.llm,
        targetEndpoint: 'http://insecure.local',
        maxPayloadBytes: 4096,
        issuedAtEpochSecs: 200,
        expiresAtEpochSecs: 100,
        userAcknowledged: false,
      );
      expect(controller.grantCloudConsent(invalidToken), isFalse);
    });
  });

  group('DashboardScreen Widget & Accessibility Tests', () {
    testWidgets('Renders all 7 independent modality adapters and enforces privacy toggles',
        (WidgetTester tester) async {
      final controller = PlatformController();
      await tester.pumpWidget(MiniAiApp(controller: controller));

      expect(
        find.text('MiniAI — Local-First Multimodal Platform'),
        findsOneWidget,
      );
      expect(find.text('LOCAL-ONLY PRIVACY'), findsOneWidget);

      // Verify modality cards exist
      for (final modality in Modality.values) {
        expect(
          find.byKey(Key('modality_card_${modality.wireValue}')),
          findsOneWidget,
        );
      }

      // Toggle force CPU-only execution
      await tester.tap(find.byKey(const Key('force_cpu_only_switch')));
      await tester.pumpAndSettle();
      expect(controller.forceCpuOnly, isTrue);

      // Toggle Local-Only Privacy Mode off to reveal explicit consent controls
      await tester.tap(find.byKey(const Key('local_only_privacy_switch')));
      await tester.pumpAndSettle();
      expect(controller.localOnlyMode, isFalse);
      expect(find.text('HYBRID CONSENT-GATED'), findsOneWidget);
      expect(find.byKey(const Key('grant_consent_button')), findsOneWidget);

      // Select Vision modality card after scrolling it into view
      final visionCard = find.byKey(const Key('modality_card_vision'));
      await tester.ensureVisible(visionCard);
      await tester.pumpAndSettle();
      await tester.tap(visionCard);
      await tester.pumpAndSettle();
      expect(controller.selectedModality, equals(Modality.vision));

      // Grant explicit consent token for Vision
      final grantBtn = find.byKey(const Key('grant_consent_button'));
      await tester.ensureVisible(grantBtn);
      await tester.pumpAndSettle();
      await tester.tap(grantBtn);
      await tester.pumpAndSettle();
      expect(controller.activeConsent, isNotNull);
      expect(controller.activeConsent!.modality, equals(Modality.vision));

      // Revoke consent token via UI
      final revokeBtn = find.byKey(const Key('revoke_consent_button'));
      await tester.ensureVisible(revokeBtn);
      await tester.pumpAndSettle();
      await tester.tap(revokeBtn);
      await tester.pumpAndSettle();
      expect(controller.activeConsent, isNull);
    });
  });
}
