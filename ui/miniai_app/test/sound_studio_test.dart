import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/models/system_contracts.dart';
import 'package:miniai_app/src/screens/shell_screen.dart';
import 'package:miniai_app/src/screens/sound_studio_screen.dart';

void main() {
  group('Subphase 7.3.1 Sound Studio & Audio Contracts Tests', () {
    test('SpeechSegmentContract and SpeechAsrResultContract JSON round-trip', () {
      const seg = SpeechSegmentContract(
        startSecs: 0.5,
        endSecs: 2.1,
        text: 'offline speech recognition',
        confidence: 0.97,
      );
      final segJson = seg.toJson();
      final decodedSeg = SpeechSegmentContract.fromJson(segJson);
      expect(decodedSeg.startSecs, equals(0.5));
      expect(decodedSeg.endSecs, equals(2.1));
      expect(decodedSeg.text, equals('offline speech recognition'));
      expect(decodedSeg.confidence, equals(0.97));

      const asr = SpeechAsrResultContract(
        fullText: 'hello world offline speech recognition',
        confidence: 0.95,
        durationSecs: 3.5,
        sha256: 'asr1234567890abcdef',
        segments: [seg],
      );
      final asrJson = asr.toJson();
      final decodedAsr = SpeechAsrResultContract.fromJson(asrJson);
      expect(decodedAsr.fullText, equals(asr.fullText));
      expect(decodedAsr.confidence, equals(0.95));
      expect(decodedAsr.durationSecs, equals(3.5));
      expect(decodedAsr.segments.length, equals(1));
    });

    test('TTS Contracts JSON round-trip and profile parsing', () {
      for (final profile in TtsVoiceProfileContract.values) {
        expect(TtsVoiceProfileContract.fromWire(profile.wireValue), equals(profile));
        expect(profile.displayName.isNotEmpty, isTrue);
      }
      expect(TtsVoiceProfileContract.fromWire('unknown'), equals(TtsVoiceProfileContract.alloy));

      const config = TtsSynthesisConfigContract(
        voice: TtsVoiceProfileContract.onyx,
        sampleRate: 24000,
        speed: 1.2,
        pitchSemitones: -2.0,
        volumeGain: 1.1,
        seed: 123,
      );
      final configJson = config.toJson();
      final decodedConfig = TtsSynthesisConfigContract.fromJson(configJson);
      expect(decodedConfig.voice, equals(TtsVoiceProfileContract.onyx));
      expect(decodedConfig.sampleRate, equals(24000));
      expect(decodedConfig.speed, equals(1.2));
      expect(decodedConfig.pitchSemitones, equals(-2.0));
      expect(decodedConfig.volumeGain, equals(1.1));
      expect(decodedConfig.seed, equals(123));

      const output = TtsOutputContract(
        durationSecs: 4.2,
        sampleRate: 22050,
        channels: 1,
        sha256: 'tts123',
        phonemeCount: 50,
        wavBytesBase64: 'UklGRiQAAABXQVZF',
      );
      final outJson = output.toJson();
      final decodedOut = TtsOutputContract.fromJson(outJson);
      expect(decodedOut.durationSecs, equals(4.2));
      expect(decodedOut.sampleRate, equals(22050));
      expect(decodedOut.channels, equals(1));
      expect(decodedOut.phonemeCount, equals(50));
    });

    test('VoiceTurn contracts JSON round-trip', () {
      const metrics = VoiceTurnMetricsContract(
        asrLatencyMs: 35.0,
        llmTtftMs: 15.0,
        llmTotalLatencyMs: 60.0,
        ttsLatencyMs: 40.0,
        totalTurnaroundMs: 135.0,
      );
      final mJson = metrics.toJson();
      final decodedM = VoiceTurnMetricsContract.fromJson(mJson);
      expect(decodedM.totalTurnaroundMs, equals(135.0));

      const turn = VoiceTurnResultContract(
        id: 'turn-1',
        userTranscription: 'hello voice',
        agentResponseText: 'hello human',
        synthesizedAudio: TtsOutputContract(
          durationSecs: 1.5,
          sampleRate: 22050,
          channels: 1,
          sha256: 'turn_audio_sha',
          phonemeCount: 18,
        ),
        metrics: metrics,
        timestampEpochSecs: 1700000000,
      );
      final turnJson = turn.toJson();
      final decodedTurn = VoiceTurnResultContract.fromJson(turnJson);
      expect(decodedTurn.id, equals('turn-1'));
      expect(decodedTurn.userTranscription, equals('hello voice'));
      expect(decodedTurn.agentResponseText, equals('hello human'));
      expect(decodedTurn.metrics.totalTurnaroundMs, equals(135.0));
    });

    test('Music contracts JSON round-trip', () {
      const musicConfig = MusicGenConfigContract(
        prompt: 'Chill lofi piano in A Minor',
        durationSecs: 15.0,
        tempoBpm: 90,
        sampleRate: 44100,
        channels: 2,
        scaleKey: 'A Minor',
        seed: 888,
      );
      final cfgJson = musicConfig.toJson();
      final decodedCfg = MusicGenConfigContract.fromJson(cfgJson);
      expect(decodedCfg.prompt, equals('Chill lofi piano in A Minor'));
      expect(decodedCfg.durationSecs, equals(15.0));
      expect(decodedCfg.tempoBpm, equals(90));
      expect(decodedCfg.channels, equals(2));
      expect(decodedCfg.scaleKey, equals('A Minor'));
      expect(decodedCfg.seed, equals(888));

      const track = MusicTrackOutputContract(
        id: 'track-101',
        prompt: 'Chill lofi',
        durationSecs: 15.0,
        sampleRate: 44100,
        channels: 2,
        tempoBpm: 90,
        scaleKey: 'A Minor',
        sha256: 'mus_track_sha',
        numSamplesPerChannel: 661500,
        timestampEpochSecs: 1700000000,
      );
      final trackJson = track.toJson();
      final decodedTrack = MusicTrackOutputContract.fromJson(trackJson);
      expect(decodedTrack.id, equals('track-101'));
      expect(decodedTrack.tempoBpm, equals(90));
      expect(decodedTrack.numSamplesPerChannel, equals(661500));
    });
  });

  group('Subphase 7.3.1 Sound Studio Controller Tests', () {
    late PlatformController controller;

    setUp(() {
      controller = PlatformController();
    });

    test('Speech ASR: transcribeAudio updates state and emits event', () async {
      expect(controller.isTranscribingSpeech, isFalse);
      expect(controller.lastAsrResult, isNull);

      await controller.transcribeAudio();

      expect(controller.isTranscribingSpeech, isFalse);
      expect(controller.lastAsrResult, isNotNull);
      expect(controller.lastAsrResult!.fullText.isNotEmpty, isTrue);
      expect(controller.lastAsrResult!.confidence, greaterThan(0.9));
      expect(controller.events.any((e) => e.eventType == 'speech_asr_completed'), isTrue);
    });

    test('TTS: parameter setters and synthesizeSpeech', () async {
      controller.setSelectedVoiceProfile(TtsVoiceProfileContract.nova);
      controller.setTtsSpeed(1.5);
      controller.setTtsPitch(4.0);
      controller.setTtsVolumeGain(1.2);

      expect(controller.selectedVoiceProfile, equals(TtsVoiceProfileContract.nova));
      expect(controller.ttsSpeed, equals(1.5));
      expect(controller.ttsPitchSemitones, equals(4.0));
      expect(controller.ttsVolumeGain, equals(1.2));

      await controller.synthesizeSpeech('Testing speech synthesis');

      expect(controller.lastTtsOutput, isNotNull);
      expect(controller.lastTtsOutput!.phonemeCount, greaterThan(0));
      expect(controller.events.any((e) => e.eventType == 'speech_tts_completed'), isTrue);
    });

    test('TTS: empty text triggers error message', () async {
      await controller.synthesizeSpeech('   ');
      expect(controller.ttsErrorMessage, isNotNull);
    });

    test('Voice Loop: executes turn and records history', () async {
      controller.setVoiceSystemPrompt('Concise assistant');
      expect(controller.voiceSystemPrompt, equals('Concise assistant'));

      await controller.executeVoiceTurn(textQuery: 'How is offline voice working?');

      expect(controller.voiceLoopHistory.length, equals(1));
      final turn = controller.voiceLoopHistory.first;
      expect(turn.userTranscription, equals('How is offline voice working?'));
      expect(turn.agentResponseText.isNotEmpty, isTrue);
      expect(turn.metrics.totalTurnaroundMs, greaterThan(0));

      controller.clearVoiceLoopHistory();
      expect(controller.voiceLoopHistory.isEmpty, isTrue);
    });

    test('Music Studio: parameter setters, generation, selection, and deletion', () async {
      controller.setMusicPrompt('Upbeat funk in G Major');
      controller.setMusicDuration(8.0);
      controller.setMusicTempo(128);
      controller.setMusicScaleKey('G Major');
      controller.setMusicChannels(1);

      expect(controller.musicConfig.prompt, equals('Upbeat funk in G Major'));
      expect(controller.musicConfig.durationSecs, equals(8.0));
      expect(controller.musicConfig.tempoBpm, equals(128));
      expect(controller.musicConfig.scaleKey, equals('G Major'));
      expect(controller.musicConfig.channels, equals(1));

      await controller.generateMusic();

      expect(controller.musicLibrary.length, equals(1));
      expect(controller.selectedMusicTrack, isNotNull);
      expect(controller.selectedMusicTrack!.prompt, equals('Upbeat funk in G Major'));

      final trackId = controller.selectedMusicTrack!.id;
      controller.deleteMusicTrack(trackId);
      expect(controller.musicLibrary.isEmpty, isTrue);
      expect(controller.selectedMusicTrack, isNull);
    });

    test('Music Studio: empty prompt triggers error message', () async {
      controller.setMusicPrompt('   ');
      await controller.generateMusic();
      expect(controller.musicErrorMessage, isNotNull);
    });
  });

  group('Subphase 7.3.1 Sound Studio Widget Tests', () {
    late PlatformController controller;

    setUp(() {
      controller = PlatformController();
    });

    testWidgets('Desktop Navigation: ShellScreen navigates to Tab 9 Sound', (tester) async {
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('desktop_navigation_rail')), findsOneWidget);
      expect(find.byKey(const Key('nav_rail_sound')), findsOneWidget);

      await tester.ensureVisible(find.byKey(const Key('nav_rail_sound')));
      await tester.tap(find.byKey(const Key('nav_rail_sound')));
      await tester.pumpAndSettle();

      expect(controller.selectedNavigationIndex, equals(9));
      expect(find.byType(SoundStudioScreen), findsOneWidget);
      expect(find.byKey(const Key('tab_speech_voice_loop')), findsOneWidget);
      expect(find.byKey(const Key('tab_music_soundstage')), findsOneWidget);
    });

    testWidgets('Tab 1 Speech & Voice Loop: ASR, TTS, and Voice Loop interactions', (tester) async {
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SoundStudioScreen(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // 1. ASR Transcribe Speech
      expect(find.byKey(const Key('btn_transcribe_audio')), findsOneWidget);
      await tester.tap(find.byKey(const Key('btn_transcribe_audio')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('asr_full_text')), findsOneWidget);
      expect(find.byKey(const Key('asr_confidence_chip')), findsOneWidget);

      // 2. Neural TTS Synthesis
      expect(find.byKey(const Key('tts_input_field')), findsOneWidget);
      expect(find.byKey(const Key('btn_synthesize_speech')), findsOneWidget);
      await tester.ensureVisible(find.byKey(const Key('btn_synthesize_speech')));
      await tester.tap(find.byKey(const Key('btn_synthesize_speech')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('tts_output_summary')), findsOneWidget);

      // 3. Offline Voice Loop Turn
      expect(find.byKey(const Key('btn_send_voice_turn')), findsOneWidget);
      await tester.enterText(find.byKey(const Key('voice_loop_input_field')), 'Hello offline voice loop');
      await tester.ensureVisible(find.byKey(const Key('btn_send_voice_turn')));
      await tester.tap(find.byKey(const Key('btn_send_voice_turn')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('voice_loop_timeline')), findsOneWidget);
      expect(find.text('Hello offline voice loop'), findsOneWidget);
    });

    testWidgets('Tab 2 Music Studio: generates music, shows active track card and library', (tester) async {
      tester.view.physicalSize = const Size(1024, 768);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SoundStudioScreen(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Switch to Music Tab
      await tester.tap(find.byKey(const Key('tab_music_soundstage')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('music_prompt_field')), findsOneWidget);
      expect(find.byKey(const Key('slider_music_tempo')), findsOneWidget);
      expect(find.byKey(const Key('slider_music_duration')), findsOneWidget);
      expect(find.byKey(const Key('btn_generate_music')), findsOneWidget);

      // Generate Music Track
      await tester.ensureVisible(find.byKey(const Key('btn_generate_music')));
      await tester.tap(find.byKey(const Key('btn_generate_music')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('active_track_prompt')), findsOneWidget);
      expect(find.byKey(const Key('music_library_list')), findsOneWidget);

      // Delete Track
      final trackId = controller.musicLibrary.first.id;
      final deleteKey = Key('btn_delete_music_$trackId');
      expect(find.byKey(deleteKey), findsOneWidget);

      await tester.ensureVisible(find.byKey(deleteKey));
      await tester.tap(find.byKey(deleteKey));
      await tester.pumpAndSettle();

      expect(find.text('No music tracks generated yet. Click "Generate Music" above.'), findsOneWidget);
    });

    testWidgets('Mobile Navigation: ShellScreen shows nav_bar_sound on mobile layout', (tester) async {
      tester.view.physicalSize = const Size(400, 800);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() => tester.view.resetPhysicalSize());

      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('mobile_navigation_bar')), findsOneWidget);
      expect(find.byKey(const Key('nav_bar_sound')), findsOneWidget);

      await tester.tap(find.byKey(const Key('nav_bar_sound')));
      await tester.pumpAndSettle();

      expect(controller.selectedNavigationIndex, equals(9));
      expect(find.byType(SoundStudioScreen), findsOneWidget);
    });
  });
}

