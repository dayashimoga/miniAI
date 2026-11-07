import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:miniai_app/src/controllers/platform_controller.dart';
import 'package:miniai_app/src/models/system_contracts.dart';
import 'package:miniai_app/src/screens/shell_screen.dart';
import 'package:miniai_app/src/screens/video_studio_screen.dart';

void main() {
  group('Subphase 8.3.1 Video & Workflow Contracts Tests', () {
    test('VideoMetadataContract JSON round-trip', () {
      const meta = VideoMetadataContract(
        containerFormat: 'mp4',
        codec: 'h264',
        width: 1920,
        height: 1080,
        durationSecs: 12.5,
        fps: 29.97,
        totalFrames: 375,
        hasAudio: true,
        audioChannels: 2,
        audioSampleRate: 48000,
        fileSizeBytes: 15400200,
        sha256Hash: 'a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef0',
      );

      final json = meta.toJson();
      final decoded = VideoMetadataContract.fromJson(json);

      expect(decoded.containerFormat, equals('mp4'));
      expect(decoded.codec, equals('h264'));
      expect(decoded.width, equals(1920));
      expect(decoded.height, equals(1080));
      expect(decoded.durationSecs, equals(12.5));
      expect(decoded.fps, equals(29.97));
      expect(decoded.totalFrames, equals(375));
      expect(decoded.hasAudio, isTrue);
      expect(decoded.audioChannels, equals(2));
      expect(decoded.audioSampleRate, equals(48000));
      expect(decoded.fileSizeBytes, equals(15400200));
      expect(decoded.sha256Hash, equals(meta.sha256Hash));
    });

    test('VideoFrameContract JSON round-trip', () {
      const frame = VideoFrameContract(
        frameIndex: 45,
        timestampMs: 1500,
        width: 1280,
        height: 720,
        isKeyframe: true,
        sceneId: 2,
        luminanceMean: 0.62,
        sha256: 'frame_sha256_mock',
      );

      final json = frame.toJson();
      final decoded = VideoFrameContract.fromJson(json);

      expect(decoded.frameIndex, equals(45));
      expect(decoded.timestampMs, equals(1500));
      expect(decoded.width, equals(1280));
      expect(decoded.height, equals(720));
      expect(decoded.isKeyframe, isTrue);
      expect(decoded.sceneId, equals(2));
      expect(decoded.luminanceMean, equals(0.62));
      expect(decoded.sha256, equals('frame_sha256_mock'));
    });

    test('SceneSegmentContract and ActionDetectionContract JSON round-trip', () {
      const action = ActionDetectionContract(
        action: 'CameraPan',
        startMs: 1000,
        endMs: 3000,
        confidence: 0.93,
        movementMagnitude: 0.22,
      );
      final actionJson = action.toJson();
      final decodedAction = ActionDetectionContract.fromJson(actionJson);

      expect(decodedAction.action, equals('CameraPan'));
      expect(decodedAction.startMs, equals(1000));
      expect(decodedAction.endMs, equals(3000));
      expect(decodedAction.confidence, equals(0.93));
      expect(decodedAction.movementMagnitude, equals(0.22));

      const scene = SceneSegmentContract(
        sceneId: 1,
        startMs: 0,
        endMs: 2500,
        keyframeIndex: 0,
        sceneType: 'DynamicAction',
        visualSummary: 'Fast motion sequence across field.',
        detectedActions: ['RapidMotion', 'SubjectRunning'],
        confidence: 0.91,
      );
      final sceneJson = scene.toJson();
      final decodedScene = SceneSegmentContract.fromJson(sceneJson);

      expect(decodedScene.sceneId, equals(1));
      expect(decodedScene.startMs, equals(0));
      expect(decodedScene.endMs, equals(2500));
      expect(decodedScene.sceneType, equals('DynamicAction'));
      expect(decodedScene.detectedActions.length, equals(2));
      expect(decodedScene.confidence, equals(0.91));
    });

    test('VideoAnalysisContract and TemporalQaResultContract JSON round-trip', () {
      const analysis = VideoAnalysisContract(
        metadata: VideoMetadataContract(
          containerFormat: 'webm',
          codec: 'vp9',
          width: 640,
          height: 360,
          durationSecs: 4.0,
          fps: 25.0,
          totalFrames: 100,
          hasAudio: false,
          audioChannels: 0,
          audioSampleRate: 0,
          fileSizeBytes: 500000,
          sha256Hash: 'webm_hash_123',
        ),
        keyframes: [
          VideoFrameContract(
            frameIndex: 0,
            timestampMs: 0,
            width: 640,
            height: 360,
            isKeyframe: true,
            sceneId: 1,
            luminanceMean: 0.5,
            sha256: 'kf0',
          ),
        ],
        scenes: [
          SceneSegmentContract(
            sceneId: 1,
            startMs: 0,
            endMs: 4000,
            keyframeIndex: 0,
            sceneType: 'EstablishingShot',
            visualSummary: 'Opening scenery.',
            detectedActions: ['StaticHold'],
            confidence: 0.95,
          ),
        ],
        overallSummary: 'A 4s scenic clip.',
        detectedActions: [
          ActionDetectionContract(
            action: 'StaticHold',
            startMs: 0,
            endMs: 4000,
            confidence: 0.95,
            movementMagnitude: 0.02,
          ),
        ],
        processingTimeMs: 85,
      );

      final analysisJson = analysis.toJson();
      final decodedAnalysis = VideoAnalysisContract.fromJson(analysisJson);

      expect(decodedAnalysis.metadata.codec, equals('vp9'));
      expect(decodedAnalysis.keyframes.length, equals(1));
      expect(decodedAnalysis.scenes.length, equals(1));
      expect(decodedAnalysis.processingTimeMs, equals(85));

      const qa = TemporalQaResultContract(
        question: 'When is the shot static?',
        answer: 'At [00:00.000 - 00:04.000]: StaticHold',
        startMs: 0,
        endMs: 4000,
        confidence: 0.96,
        supportingSceneId: 1,
        supportingFrameIndices: [0],
      );

      final qaJson = qa.toJson();
      final decodedQa = TemporalQaResultContract.fromJson(qaJson);

      expect(decodedQa.question, equals(qa.question));
      expect(decodedQa.startMs, equals(0));
      expect(decodedQa.endMs, equals(4000));
      expect(decodedQa.confidence, equals(0.96));
      expect(decodedQa.supportingSceneId, equals(1));
    });

    test('VideoSynthesis & Workflow Contracts JSON round-trip', () {
      const synConfig = VideoSynthesisConfigContract(
        prompt: 'Futuristic highway at dusk',
        motion: 'pan_left',
        width: 640,
        height: 360,
        durationSecs: 3.0,
        fps: 24.0,
        seed: 777,
        generateAudio: true,
      );

      final synJson = synConfig.toJson();
      final decodedSyn = VideoSynthesisConfigContract.fromJson(synJson);

      expect(decodedSyn.prompt, equals(synConfig.prompt));
      expect(decodedSyn.motion, equals('pan_left'));
      expect(decodedSyn.durationSecs, equals(3.0));
      expect(decodedSyn.seed, equals(777));
      expect(decodedSyn.generateAudio, isTrue);

      const node = WorkflowNodeContract(
        id: 'node_test',
        name: 'Test Node',
        nodeType: 'llm_prompt',
        status: 'completed',
        durationMs: 42,
        outputSummary: 'Tokens emitted',
      );
      final nodeJson = node.toJson();
      final decodedNode = WorkflowNodeContract.fromJson(nodeJson);

      expect(decodedNode.id, equals('node_test'));
      expect(decodedNode.status, equals('completed'));
      expect(decodedNode.durationMs, equals(42));

      const wf = WorkflowExecutionResultContract(
        graphId: 'test_graph',
        totalLatencyMs: 120,
        nodeTelemetry: [node],
      );
      final wfJson = wf.toJson();
      final decodedWf = WorkflowExecutionResultContract.fromJson(wfJson);

      expect(decodedWf.graphId, equals('test_graph'));
      expect(decodedWf.totalLatencyMs, equals(120));
      expect(decodedWf.nodeTelemetry.length, equals(1));
    });
  });

  group('Subphase 8.3.1 Video Controller Tests', () {
    late PlatformController controller;

    setUp(() {
      controller = PlatformController();
    });

    test('loadSampleVideo populates analysis, resets timeline and emits event', () {
      expect(controller.activeVideoAnalysis, isNull);
      expect(controller.videoTimelinePositionMs, equals(0.0));

      controller.loadSampleVideo();

      expect(controller.activeVideoAnalysis, isNotNull);
      expect(controller.activeVideoAnalysis!.metadata.codec, equals('h264'));
      expect(controller.activeVideoAnalysis!.keyframes.length, equals(4));
      expect(controller.activeVideoAnalysis!.scenes.length, equals(3));
      expect(controller.selectedVideoKeyframe, isNotNull);
      expect(controller.selectedVideoKeyframe!.frameIndex, equals(0));
      expect(
        controller.events.any((e) => e.eventType == 'video_sample_loaded'),
        isTrue,
      );
    });

    test('seekVideoTimeline clamps position and updates selected keyframe', () {
      controller.loadSampleVideo();

      // Seek to 2100ms (near frame 60 @ 2000ms)
      controller.seekVideoTimeline(2100.0);
      expect(controller.videoTimelinePositionMs, equals(2100.0));
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(60));

      // Seek past end (clamp to 6000ms)
      controller.seekVideoTimeline(9999.0);
      expect(controller.videoTimelinePositionMs, equals(6000.0));
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(180));

      // Seek before start (clamp to 0ms)
      controller.seekVideoTimeline(-500.0);
      expect(controller.videoTimelinePositionMs, equals(0.0));
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(0));
    });

    test('toggleVideoPlayPause flips state', () {
      expect(controller.isVideoPlaying, isFalse);
      controller.toggleVideoPlayPause();
      expect(controller.isVideoPlaying, isTrue);
      controller.toggleVideoPlayPause();
      expect(controller.isVideoPlaying, isFalse);
    });

    test('selectPreviousKeyframe and selectNextKeyframe traverse frames', () {
      controller.loadSampleVideo();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(0));

      controller.selectNextKeyframe();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(60));
      expect(controller.videoTimelinePositionMs, equals(2000.0));

      controller.selectNextKeyframe();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(120));

      controller.selectNextKeyframe();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(180));

      // At end, stays at 180
      controller.selectNextKeyframe();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(180));

      controller.selectPreviousKeyframe();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(120));

      controller.selectPreviousKeyframe();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(60));

      controller.selectPreviousKeyframe();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(0));

      // At beginning, stays at 0
      controller.selectPreviousKeyframe();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(0));
    });

    test('askVideoQuestion handles error cases and grounded answers', () {
      // No video loaded
      controller.askVideoQuestion('Any action?');
      expect(controller.videoErrorMessage, contains('Please load or analyze'));

      controller.loadSampleVideo();

      // Empty query
      controller.askVideoQuestion('   ');
      expect(controller.videoErrorMessage, contains('Please enter a video question'));

      // Valid action query
      controller.askVideoQuestion('Where does rapid action occur?');
      expect(controller.videoErrorMessage, isNull);
      expect(controller.activeTemporalQaResult, isNotNull);
      expect(controller.activeTemporalQaResult!.supportingSceneId, equals(2));
      expect(controller.activeTemporalQaResult!.startMs, equals(2000));
      expect(controller.videoTimelinePositionMs, equals(2000.0));
      expect(
        controller.events.any((e) => e.eventType == 'video_temporal_qa'),
        isTrue,
      );

      // Cabin scene query
      controller.askVideoQuestion('Show the wooden cabin at the end');
      expect(controller.activeTemporalQaResult!.supportingSceneId, equals(3));
      expect(controller.activeTemporalQaResult!.startMs, equals(4000));
    });

    test('updateVideoSynthesisConfig and synthesizeVideo', () async {
      controller.updateVideoSynthesisConfig(
        prompt: 'Volcanic eruption aerial',
        motion: 'zoom_in',
        durationSecs: 4.0,
        width: 640,
        height: 360,
        generateAudio: true,
      );

      expect(controller.videoSynthesisConfig.prompt, equals('Volcanic eruption aerial'));
      expect(controller.videoSynthesisConfig.motion, equals('zoom_in'));
      expect(controller.videoSynthesisConfig.durationSecs, equals(4.0));
      expect(controller.videoSynthesisConfig.width, equals(640));

      // Error on empty prompt
      controller.updateVideoSynthesisConfig(prompt: '  ');
      await controller.synthesizeVideo();
      expect(controller.videoErrorMessage, contains('Please enter a video prompt'));

      controller.updateVideoSynthesisConfig(prompt: 'Volcanic eruption aerial');
      await controller.synthesizeVideo();

      expect(controller.activeSynthesizedVideo, isNotNull);
      expect(controller.activeSynthesizedVideo!.metadata.width, equals(640));
      expect(controller.activeSynthesizedVideo!.metadata.height, equals(360));
      expect(controller.activeSynthesizedVideo!.hasAudio, isTrue);
      expect(
        controller.events.any((e) => e.eventType == 'video_synthesis_completed'),
        isTrue,
      );
    });

    test('executeMediaWorkflow runs DAG nodes and emits telemetry', () async {
      controller.setSelectedWorkflowPreset('Storyboard & Voice (LLM -> TTS -> Music)');
      expect(
        controller.selectedWorkflowPreset,
        equals('Storyboard & Voice (LLM -> TTS -> Music)'),
      );

      await controller.executeMediaWorkflow();

      expect(controller.isExecutingWorkflow, isFalse);
      expect(controller.activeWorkflowResult, isNotNull);
      expect(controller.activeWorkflowResult!.nodeTelemetry.length, equals(6));
      expect(
        controller.activeWorkflowResult!.nodeTelemetry.every((n) => n.status == 'completed'),
        isTrue,
      );
      expect(controller.activeWorkflowResult!.totalLatencyMs, greaterThan(0));
      expect(
        controller.events.any((e) => e.eventType == 'multimodal_workflow_completed'),
        isTrue,
      );
    });
  });

  group('Subphase 8.3.1 Video Studio Widget Tests', () {
    late PlatformController controller;

    setUp(() {
      controller = PlatformController();
    });

    testWidgets('renders empty state and loads sample video', (tester) async {
      tester.view.physicalSize = const Size(1280, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: VideoStudioScreen(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Check Tabs
      expect(find.byKey(const Key('video_studio_tab_bar')), findsOneWidget);
      expect(find.byKey(const Key('tab_video_timeline')), findsOneWidget);
      expect(find.byKey(const Key('tab_video_synthesis_workflow')), findsOneWidget);

      // Check empty state
      expect(find.textContaining('No video loaded yet'), findsOneWidget);
      expect(find.byKey(const Key('load_sample_video_button')), findsOneWidget);

      // Tap load sample video
      await tester.tap(find.byKey(const Key('load_sample_video_button')));
      await tester.pumpAndSettle();

      // Player card, gallery, scenes, QA cards should now be rendered
      expect(find.byKey(const Key('video_player_card')), findsOneWidget);
      expect(find.byKey(const Key('keyframe_gallery_strip')), findsOneWidget);
      expect(find.byKey(const Key('scene_segments_card')), findsOneWidget);
      expect(find.byKey(const Key('video_qa_card')), findsOneWidget);

      // Keyframes
      expect(find.byKey(const Key('keyframe_card_0')), findsOneWidget);
      expect(find.byKey(const Key('keyframe_card_60')), findsOneWidget);
      expect(find.byKey(const Key('keyframe_card_120')), findsOneWidget);
      expect(find.byKey(const Key('keyframe_card_180')), findsOneWidget);
    });

    testWidgets('controls video playback, seeking, and next/prev keyframes', (tester) async {
      tester.view.physicalSize = const Size(1280, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);

      controller.loadSampleVideo();

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: VideoStudioScreen(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Toggle play/pause
      expect(controller.isVideoPlaying, isFalse);
      await tester.tap(find.byKey(const Key('play_pause_button')));
      await tester.pumpAndSettle();
      expect(controller.isVideoPlaying, isTrue);

      await tester.tap(find.byKey(const Key('play_pause_button')));
      await tester.pumpAndSettle();
      expect(controller.isVideoPlaying, isFalse);

      // Next keyframe
      await tester.tap(find.byKey(const Key('next_keyframe_button')));
      await tester.pumpAndSettle();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(60));

      // Prev keyframe
      await tester.tap(find.byKey(const Key('prev_keyframe_button')));
      await tester.pumpAndSettle();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(0));

      // Tap keyframe card 120
      await tester.tap(find.byKey(const Key('keyframe_card_120')));
      await tester.pumpAndSettle();
      expect(controller.selectedVideoKeyframe?.frameIndex, equals(120));
      expect(controller.videoTimelinePositionMs, equals(4000.0));
    });

    testWidgets('submits temporal video QA and displays grounded timestamp result', (tester) async {
      tester.view.physicalSize = const Size(1280, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);

      controller.loadSampleVideo();

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: VideoStudioScreen(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Submit QA
      await tester.ensureVisible(find.byKey(const Key('video_qa_submit_button')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('video_qa_submit_button')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('video_qa_result')), findsOneWidget);
      expect(find.textContaining('Temporal Grounding: [2000ms - 4000ms]'), findsOneWidget);
      expect(find.textContaining('DynamicAction'), findsWidgets);
    });

    testWidgets('interacts with video synthesis and DAG workflow execution', (tester) async {
      tester.view.physicalSize = const Size(1280, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: VideoStudioScreen(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Switch to Synthesis & Workflow tab
      await tester.tap(find.byKey(const Key('tab_video_synthesis_workflow')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('video_synthesis_card')), findsOneWidget);
      expect(find.byKey(const Key('workflow_dag_card')), findsOneWidget);

      // Generate Video
      await tester.tap(find.byKey(const Key('generate_video_button')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('synthesis_result_card')), findsOneWidget);
      expect(find.textContaining('Synthesized Video Ready'), findsOneWidget);

      // Execute Workflow
      await tester.ensureVisible(find.byKey(const Key('execute_workflow_button')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('execute_workflow_button')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('workflow_node_monitor')), findsOneWidget);
      expect(find.text('LLM Storyboard Director'), findsOneWidget);
      expect(find.text('Multimedia Multiplexer (Mux)'), findsOneWidget);
      expect(find.textContaining('Total Latency: 196 ms'), findsOneWidget);
    });

    testWidgets('Desktop Navigation: ShellScreen routes to VideoStudioScreen via nav_rail_video', (tester) async {
      tester.view.physicalSize = const Size(1280, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);

      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('desktop_navigation_rail')), findsOneWidget);
      await tester.ensureVisible(find.byKey(const Key('nav_rail_video')));
      await tester.tap(find.byKey(const Key('nav_rail_video')));
      await tester.pumpAndSettle();

      expect(controller.selectedNavigationIndex, equals(10));
      expect(find.byType(VideoStudioScreen), findsOneWidget);
    });

    testWidgets('Mobile Navigation: ShellScreen routes to VideoStudioScreen via nav_bar_video', (tester) async {
      tester.view.physicalSize = const Size(600, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);

      await tester.pumpWidget(
        MaterialApp(
          home: ShellScreen(controller: controller),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('mobile_navigation_bar')), findsOneWidget);
      expect(find.byKey(const Key('nav_bar_video')), findsOneWidget);

      await tester.tap(find.byKey(const Key('nav_bar_video')));
      await tester.pumpAndSettle();

      expect(controller.selectedNavigationIndex, equals(10));
      expect(find.byType(VideoStudioScreen), findsOneWidget);
    });
  });
}
