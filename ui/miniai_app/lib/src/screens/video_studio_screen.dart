import 'package:flutter/material.dart';
import '../controllers/platform_controller.dart';
import '../models/system_contracts.dart';

/// Video Studio Screen providing Video Understanding, Keyframe Sampling,
/// Temporal Scene Segmentation, Grounded Video QA, Generative Video Synthesis,
/// and Multimodal DAG Workflow execution.
class VideoStudioScreen extends StatefulWidget {
  const VideoStudioScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  State<VideoStudioScreen> createState() => _VideoStudioScreenState();
}

class _VideoStudioScreenState extends State<VideoStudioScreen> {
  late final TextEditingController _qaController;
  late final TextEditingController _promptController;

  @override
  void initState() {
    super.initState();
    _qaController = TextEditingController(
      text: 'Where does rapid action movement occur in the video?',
    );
    _promptController = TextEditingController(
      text: widget.controller.videoSynthesisConfig.prompt,
    );
  }

  @override
  void dispose() {
    _qaController.dispose();
    _promptController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        return DefaultTabController(
          length: 2,
          child: Scaffold(
            appBar: const TabBar(
              key: Key('video_studio_tab_bar'),
              tabs: [
                Tab(
                  key: Key('tab_video_timeline'),
                  icon: Icon(Icons.video_library),
                  text: 'Timeline & Scene QA',
                ),
                Tab(
                  key: Key('tab_video_synthesis_workflow'),
                  icon: Icon(Icons.auto_awesome_motion),
                  text: 'Synthesis & DAG Workflow',
                ),
              ],
            ),
            body: TabBarView(
              children: [
                _buildTimelineAndQaTab(context),
                _buildSynthesisAndWorkflowTab(context),
              ],
            ),
          ),
        );
      },
    );
  }

  // ===========================================================================
  // Tab 1: Video Timeline & Scene Understanding
  // ===========================================================================

  Widget _buildTimelineAndQaTab(BuildContext context) {
    final controller = widget.controller;
    final analysis = controller.activeVideoAnalysis;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header Row with Actions
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Video Understanding & Temporal Timeline',
                      style: Theme.of(context).textTheme.titleLarge?.copyWith(
                            fontWeight: FontWeight.bold,
                          ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'Sample-rate keyframe extraction, scene segmentation, and temporal grounding.',
                      style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                            color: Theme.of(context).colorScheme.outline,
                          ),
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 12),
              FilledButton.icon(
                key: const Key('load_sample_video_button'),
                icon: const Icon(Icons.file_open),
                label: const Text('Load Sample Video'),
                onPressed: () {
                  controller.loadSampleVideo();
                },
              ),
            ],
          ),
          const SizedBox(height: 16),

          if (controller.videoErrorMessage != null) ...[
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.errorContainer,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                children: [
                  Icon(
                    Icons.error_outline,
                    color: Theme.of(context).colorScheme.error,
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      controller.videoErrorMessage!,
                      style: TextStyle(
                        color: Theme.of(context).colorScheme.onErrorContainer,
                      ),
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
          ],

          if (analysis == null)
            Card(
              child: Padding(
                padding: const EdgeInsets.all(32),
                child: Center(
                  child: Column(
                    children: [
                      Icon(
                        Icons.movie_creation_outlined,
                        size: 48,
                        color: Theme.of(context).colorScheme.outline,
                      ),
                      const SizedBox(height: 12),
                      const Text(
                        'No video loaded yet. Click "Load Sample Video" to inspect frames, scenes, and run temporal QA.',
                      ),
                    ],
                  ),
                ),
              ),
            )
          else ...[
            // Video Player & Scrubber Card
            _buildVideoPlayerCard(context, analysis),
            const SizedBox(height: 16),

            // Keyframe Gallery Strip
            _buildKeyframeGalleryCard(context, analysis),
            const SizedBox(height: 16),

            // Scene Segments Card
            _buildSceneSegmentsCard(context, analysis),
            const SizedBox(height: 16),

            // Temporal Video QA Card
            _buildTemporalQaCard(context, analysis),
          ],
        ],
      ),
    );
  }

  Widget _buildVideoPlayerCard(
    BuildContext context,
    VideoAnalysisContract analysis,
  ) {
    final controller = widget.controller;
    final totalDurationMs = analysis.metadata.durationSecs * 1000.0;
    final currentPosMs = controller.videoTimelinePositionMs;
    final activeKf = controller.selectedVideoKeyframe;

    return Card(
      key: const Key('video_player_card'),
      elevation: 2,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Video Frame Preview Canvas
            Container(
              height: 220,
              width: double.infinity,
              decoration: BoxDecoration(
                color: Colors.black87,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: Theme.of(context).colorScheme.outlineVariant,
                ),
              ),
              child: Stack(
                children: [
                  Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(
                          controller.isVideoPlaying
                              ? Icons.slow_motion_video
                              : Icons.image,
                          size: 56,
                          color: Colors.white70,
                        ),
                        const SizedBox(height: 8),
                        Text(
                          activeKf != null
                              ? 'Keyframe #${activeKf.frameIndex} @ ${activeKf.timestampMs} ms (Scene ${activeKf.sceneId})'
                              : 'Video Playback Stream',
                          style: const TextStyle(
                            color: Colors.white,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                        const SizedBox(height: 4),
                        Text(
                          '${analysis.metadata.width}x${analysis.metadata.height} | ${analysis.metadata.fps.toStringAsFixed(1)} FPS | Codec: ${analysis.metadata.codec.toUpperCase()}',
                          style: const TextStyle(
                            color: Colors.white60,
                            fontSize: 12,
                          ),
                        ),
                      ],
                    ),
                  ),
                  Positioned(
                    top: 12,
                    left: 12,
                    child: Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 4,
                      ),
                      decoration: BoxDecoration(
                        color: Colors.black54,
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        'Time: ${(currentPosMs / 1000.0).toStringAsFixed(2)}s / ${(totalDurationMs / 1000.0).toStringAsFixed(2)}s',
                        style: const TextStyle(
                          color: Colors.white,
                          fontSize: 12,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ),
                  ),
                  Positioned(
                    top: 12,
                    right: 12,
                    child: Chip(
                      label: Text(
                        analysis.metadata.hasAudio ? 'AUDIO ON' : 'NO AUDIO',
                        style: const TextStyle(fontSize: 11),
                      ),
                      visualDensity: VisualDensity.compact,
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 12),

            // Controls & Scrubber
            Row(
              children: [
                IconButton(
                  key: const Key('prev_keyframe_button'),
                  tooltip: 'Previous Keyframe',
                  icon: const Icon(Icons.skip_previous),
                  onPressed: () {
                    controller.selectPreviousKeyframe();
                  },
                ),
                IconButton(
                  key: const Key('play_pause_button'),
                  tooltip: controller.isVideoPlaying ? 'Pause' : 'Play',
                  icon: Icon(
                    controller.isVideoPlaying
                        ? Icons.pause_circle_filled
                        : Icons.play_circle_filled,
                    size: 32,
                    color: Theme.of(context).colorScheme.primary,
                  ),
                  onPressed: () {
                    controller.toggleVideoPlayPause();
                  },
                ),
                IconButton(
                  key: const Key('next_keyframe_button'),
                  tooltip: 'Next Keyframe',
                  icon: const Icon(Icons.skip_next),
                  onPressed: () {
                    controller.selectNextKeyframe();
                  },
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Slider(
                    key: const Key('timeline_scrubber_slider'),
                    min: 0.0,
                    max: totalDurationMs > 0 ? totalDurationMs : 1.0,
                    value: currentPosMs.clamp(
                      0.0,
                      totalDurationMs > 0 ? totalDurationMs : 1.0,
                    ),
                    onChanged: (val) {
                      controller.seekVideoTimeline(val);
                    },
                  ),
                ),
                Text(
                  '${(currentPosMs / 1000.0).toStringAsFixed(1)}s / ${(totalDurationMs / 1000.0).toStringAsFixed(1)}s',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildKeyframeGalleryCard(
    BuildContext context,
    VideoAnalysisContract analysis,
  ) {
    final controller = widget.controller;
    final selectedKf = controller.selectedVideoKeyframe;

    return Card(
      key: const Key('keyframe_gallery_strip'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Keyframe Gallery (${analysis.keyframes.length} frames)',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                ),
                Text(
                  'Click a keyframe to jump timeline',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(context).colorScheme.outline,
                      ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: analysis.keyframes.map((kf) {
                  final isSelected = selectedKf?.frameIndex == kf.frameIndex;
                  return Padding(
                    padding: const EdgeInsets.only(right: 12),
                    child: InkWell(
                      key: Key('keyframe_card_${kf.frameIndex}'),
                      borderRadius: BorderRadius.circular(8),
                      onTap: () {
                        controller.seekVideoTimeline(kf.timestampMs.toDouble());
                      },
                      child: Container(
                        width: 130,
                        padding: const EdgeInsets.all(8),
                        decoration: BoxDecoration(
                          color: isSelected
                              ? Theme.of(context).colorScheme.primaryContainer
                              : Theme.of(context).colorScheme.surfaceContainerHighest,
                          border: Border.all(
                            color: isSelected
                                ? Theme.of(context).colorScheme.primary
                                : Colors.transparent,
                            width: 2,
                          ),
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Container(
                              height: 60,
                              width: double.infinity,
                              color: Colors.black87,
                              child: Center(
                                child: Icon(
                                  Icons.image,
                                  color: isSelected
                                      ? Theme.of(context).colorScheme.primary
                                      : Colors.white54,
                                ),
                              ),
                            ),
                            const SizedBox(height: 6),
                            Text(
                              'Frame #${kf.frameIndex}',
                              style: const TextStyle(
                                fontWeight: FontWeight.bold,
                                fontSize: 12,
                              ),
                            ),
                            Text(
                              '${kf.timestampMs} ms',
                              style: TextStyle(
                                fontSize: 11,
                                color: Theme.of(context).colorScheme.outline,
                              ),
                            ),
                            Text(
                              'Scene #${kf.sceneId}',
                              style: TextStyle(
                                fontSize: 11,
                                color: Theme.of(context).colorScheme.primary,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  );
                }).toList(),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSceneSegmentsCard(
    BuildContext context,
    VideoAnalysisContract analysis,
  ) {
    final controller = widget.controller;

    return Card(
      key: const Key('scene_segments_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Temporal Scene Segments (${analysis.scenes.length} scenes)',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                ),
                Chip(
                  label: Text('Inference: ${analysis.processingTimeMs} ms'),
                  visualDensity: VisualDensity.compact,
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              analysis.overallSummary,
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            const SizedBox(height: 12),
            ...analysis.scenes.map((scene) {
              return Container(
                margin: const EdgeInsets.only(bottom: 8),
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.surfaceContainerLow,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(
                    color: Theme.of(context).colorScheme.outlineVariant,
                  ),
                ),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 4,
                      ),
                      decoration: BoxDecoration(
                        color: Theme.of(context).colorScheme.secondaryContainer,
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        'Scene ${scene.sceneId}',
                        style: TextStyle(
                          fontWeight: FontWeight.bold,
                          color: Theme.of(context)
                              .colorScheme
                              .onSecondaryContainer,
                          fontSize: 12,
                        ),
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              Text(
                                '${scene.sceneType} (${scene.startMs}ms - ${scene.endMs}ms)',
                                style: const TextStyle(
                                  fontWeight: FontWeight.bold,
                                ),
                              ),
                              const Spacer(),
                              Text(
                                '${(scene.confidence * 100).toStringAsFixed(1)}% conf',
                                style: TextStyle(
                                  fontSize: 11,
                                  color: Theme.of(context).colorScheme.outline,
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: 4),
                          Text(
                            scene.visualSummary,
                            style: Theme.of(context).textTheme.bodyMedium,
                          ),
                          const SizedBox(height: 6),
                          Wrap(
                            spacing: 6,
                            runSpacing: 4,
                            children: scene.detectedActions.map((act) {
                              return Chip(
                                label: Text(act, style: const TextStyle(fontSize: 11)),
                                visualDensity: VisualDensity.compact,
                              );
                            }).toList(),
                          ),
                        ],
                      ),
                    ),
                    IconButton(
                      tooltip: 'Jump to scene start',
                      icon: const Icon(Icons.arrow_forward_ios, size: 16),
                      onPressed: () {
                        controller.seekVideoTimeline(scene.startMs.toDouble());
                      },
                    ),
                  ],
                ),
              );
            }),
          ],
        ),
      ),
    );
  }

  Widget _buildTemporalQaCard(
    BuildContext context,
    VideoAnalysisContract analysis,
  ) {
    final controller = widget.controller;
    final qaResult = controller.activeTemporalQaResult;

    return Card(
      key: const Key('video_qa_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Grounded Temporal Video QA',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 4),
            Text(
              'Ask natural language queries about video events with millisecond-exact temporal timestamp grounding.',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.outline,
                  ),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    key: const Key('video_qa_input'),
                    controller: _qaController,
                    decoration: const InputDecoration(
                      labelText: 'Question about video content',
                      hintText: 'e.g. When does rapid camera motion take place?',
                      border: OutlineInputBorder(),
                    ),
                    onSubmitted: (val) {
                      controller.askVideoQuestion(val);
                    },
                  ),
                ),
                const SizedBox(width: 12),
                FilledButton.icon(
                  key: const Key('video_qa_submit_button'),
                  icon: const Icon(Icons.question_answer),
                  label: const Text('Ask Video'),
                  onPressed: () {
                    controller.askVideoQuestion(_qaController.text);
                  },
                ),
              ],
            ),
            if (qaResult != null) ...[
              const SizedBox(height: 16),
              Container(
                key: const Key('video_qa_result'),
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.tertiaryContainer,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(
                          'Temporal Grounding: [${qaResult.startMs}ms - ${qaResult.endMs}ms]',
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            color: Theme.of(context)
                                .colorScheme
                                .onTertiaryContainer,
                          ),
                        ),
                        Chip(
                          label: Text(
                            '${(qaResult.confidence * 100).toStringAsFixed(1)}% conf',
                            style: const TextStyle(fontSize: 11),
                          ),
                          visualDensity: VisualDensity.compact,
                        ),
                      ],
                    ),
                    const SizedBox(height: 6),
                    Text(
                      qaResult.answer,
                      style: TextStyle(
                        color: Theme.of(context).colorScheme.onTertiaryContainer,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  // ===========================================================================
  // Tab 2: Video Synthesis & Multimodal DAG Workflows
  // ===========================================================================

  Widget _buildSynthesisAndWorkflowTab(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildVideoSynthesisSection(context),
          const SizedBox(height: 24),
          const Divider(),
          const SizedBox(height: 16),
          _buildWorkflowDagSection(context),
        ],
      ),
    );
  }

  Widget _buildVideoSynthesisSection(BuildContext context) {
    final controller = widget.controller;
    final config = controller.videoSynthesisConfig;
    final synVideo = controller.activeSynthesizedVideo;

    return Card(
      key: const Key('video_synthesis_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Generative Video Synthesis & Camera Motion',
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 4),
            Text(
              'Text-to-video and image-to-video interpolation with parameterized camera profiles.',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.outline,
                  ),
            ),
            const SizedBox(height: 16),

            // Prompt Input
            TextField(
              key: const Key('video_prompt_input'),
              controller: _promptController,
              decoration: const InputDecoration(
                labelText: 'Video Generation Prompt',
                hintText: 'Cinematic drone shot flying across emerald fjord...',
                border: OutlineInputBorder(),
              ),
              onChanged: (val) {
                controller.updateVideoSynthesisConfig(prompt: val);
              },
            ),
            const SizedBox(height: 16),

            // Motion & Resolution Dropdowns
            Row(
              children: [
                Expanded(
                  child: DropdownButtonFormField<String>(
                    key: const Key('camera_motion_dropdown'),
                    isExpanded: true,
                    decoration: const InputDecoration(
                      labelText: 'Camera Motion Profile',
                      border: OutlineInputBorder(),
                    ),
                    initialValue: config.motion,
                    items: const [
                      DropdownMenuItem(value: 'pan_left', child: Text('Pan Left')),
                      DropdownMenuItem(value: 'pan_right', child: Text('Pan Right')),
                      DropdownMenuItem(value: 'zoom_in', child: Text('Zoom In')),
                      DropdownMenuItem(value: 'zoom_out', child: Text('Zoom Out')),
                      DropdownMenuItem(value: 'tilt_up', child: Text('Tilt Up')),
                      DropdownMenuItem(value: 'tilt_down', child: Text('Tilt Down')),
                      DropdownMenuItem(value: 'static', child: Text('Static Hold')),
                      DropdownMenuItem(value: 'orbit', child: Text('Orbit')),
                    ],
                    onChanged: (val) {
                      if (val != null) {
                        controller.updateVideoSynthesisConfig(motion: val);
                      }
                    },
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: DropdownButtonFormField<String>(
                    key: const Key('video_resolution_dropdown'),
                    isExpanded: true,
                    decoration: const InputDecoration(
                      labelText: 'Resolution',
                      border: OutlineInputBorder(),
                    ),
                    initialValue: '${config.width}x${config.height}',
                    items: const [
                      DropdownMenuItem(value: '320x240', child: Text('320x240 (Fast CPU)')),
                      DropdownMenuItem(value: '640x360', child: Text('640x360 (Standard)')),
                      DropdownMenuItem(value: '1280x720', child: Text('1280x720 (HD)')),
                    ],
                    onChanged: (val) {
                      if (val != null) {
                        final parts = val.split('x');
                        controller.updateVideoSynthesisConfig(
                          width: int.parse(parts[0]),
                          height: int.parse(parts[1]),
                        );
                      }
                    },
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),

            // Duration Slider
            Row(
              children: [
                Text(
                  'Duration: ${config.durationSecs.toStringAsFixed(1)}s',
                  style: const TextStyle(fontWeight: FontWeight.bold),
                ),
                Expanded(
                  child: Slider(
                    key: const Key('video_duration_slider'),
                    min: 1.0,
                    max: 10.0,
                    divisions: 18,
                    value: config.durationSecs,
                    onChanged: (val) {
                      controller.updateVideoSynthesisConfig(durationSecs: val);
                    },
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),

            // Generate Button
            FilledButton.icon(
              key: const Key('generate_video_button'),
              icon: controller.isSynthesizingVideo
                  ? const SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: Colors.white,
                      ),
                    )
                  : const Icon(Icons.video_call),
              label: Text(
                controller.isSynthesizingVideo
                    ? 'Synthesizing Video (${(controller.videoSynthesisProgress * 100).toInt()}%)...'
                    : 'Generate Video',
              ),
              onPressed: controller.isSynthesizingVideo
                  ? null
                  : () {
                      controller.updateVideoSynthesisConfig(
                        prompt: _promptController.text,
                      );
                      controller.synthesizeVideo();
                    },
            ),

            if (controller.isSynthesizingVideo) ...[
              const SizedBox(height: 12),
              LinearProgressIndicator(
                value: controller.videoSynthesisProgress,
              ),
            ],

            if (synVideo != null) ...[
              const SizedBox(height: 16),
              Container(
                key: const Key('synthesis_result_card'),
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.primaryContainer,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Synthesized Video Ready (${synVideo.metadata.width}x${synVideo.metadata.height}, ${synVideo.metadata.fps} FPS)',
                      style: TextStyle(
                        fontWeight: FontWeight.bold,
                        color: Theme.of(context).colorScheme.onPrimaryContainer,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'Frames: ${synVideo.metadata.totalFrames} | Duration: ${synVideo.metadata.durationSecs}s | Audio: ${synVideo.hasAudio ? "Synchronized WAV" : "None"}',
                      style: TextStyle(
                        fontSize: 12,
                        color: Theme.of(context).colorScheme.onPrimaryContainer,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'SHA-256: ${synVideo.metadata.sha256Hash}',
                      style: TextStyle(
                        fontSize: 11,
                        fontFamily: 'monospace',
                        color: Theme.of(context).colorScheme.onPrimaryContainer,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildWorkflowDagSection(BuildContext context) {
    final controller = widget.controller;
    final wfResult = controller.activeWorkflowResult;

    final presets = [
      'Full Multimodal Cinematic (LLM -> Image -> Video -> TTS -> Music -> Mux)',
      'Storyboard & Voice (LLM -> TTS -> Music)',
      'Visual Concept Animation (Image -> Video)',
    ];

    return Card(
      key: const Key('workflow_dag_card'),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Multimodal DAG Workflow Orchestration',
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 4),
            Text(
              'Topologically sorted asynchronous pipelines connecting LLM, Vision, Image, Speech, Music, and Video.',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.outline,
                  ),
            ),
            const SizedBox(height: 16),

            // Preset Dropdown
            DropdownButtonFormField<String>(
              key: const Key('workflow_preset_dropdown'),
              isExpanded: true,
              decoration: const InputDecoration(
                labelText: 'Workflow Pipeline Preset',
                border: OutlineInputBorder(),
              ),
              initialValue: controller.selectedWorkflowPreset,
              items: presets.map((p) {
                return DropdownMenuItem(
                  value: p,
                  child: Text(p, overflow: TextOverflow.ellipsis),
                );
              }).toList(),
              onChanged: (val) {
                if (val != null) {
                  controller.setSelectedWorkflowPreset(val);
                }
              },
            ),
            const SizedBox(height: 16),

            // Execute Workflow Button
            FilledButton.icon(
              key: const Key('execute_workflow_button'),
              icon: controller.isExecutingWorkflow
                  ? const SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: Colors.white,
                      ),
                    )
                  : const Icon(Icons.account_tree),
              label: Text(
                controller.isExecutingWorkflow
                    ? 'Executing Workflow Pipeline...'
                    : 'Execute Media Workflow',
              ),
              onPressed: controller.isExecutingWorkflow
                  ? null
                  : () {
                      controller.executeMediaWorkflow();
                    },
            ),

            if (wfResult != null) ...[
              const SizedBox(height: 16),
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.surfaceContainerHighest,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Text(
                      'Graph: ${wfResult.graphId}',
                      style: const TextStyle(fontWeight: FontWeight.bold),
                    ),
                    Chip(
                      label: Text('Total Latency: ${wfResult.totalLatencyMs} ms'),
                      visualDensity: VisualDensity.compact,
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 12),

              // Node Monitor
              Column(
                key: const Key('workflow_node_monitor'),
                children: wfResult.nodeTelemetry.map((node) {
                  Color statusColor;
                  IconData statusIcon;

                  switch (node.status) {
                    case 'completed':
                      statusColor = Colors.green;
                      statusIcon = Icons.check_circle;
                      break;
                    case 'running':
                      statusColor = Colors.orange;
                      statusIcon = Icons.refresh;
                      break;
                    case 'failed':
                      statusColor = Colors.red;
                      statusIcon = Icons.error;
                      break;
                    default:
                      statusColor = Colors.grey;
                      statusIcon = Icons.pending;
                  }

                  return Container(
                    margin: const EdgeInsets.only(bottom: 8),
                    padding: const EdgeInsets.all(10),
                    decoration: BoxDecoration(
                      color: Theme.of(context).colorScheme.surfaceContainerLow,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(
                        color: Theme.of(context).colorScheme.outlineVariant,
                      ),
                    ),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Icon(statusIcon, color: statusColor, size: 20),
                        const SizedBox(width: 10),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Row(
                                children: [
                                  Text(
                                    node.name,
                                    style: const TextStyle(
                                      fontWeight: FontWeight.bold,
                                      fontSize: 13,
                                    ),
                                  ),
                                  const SizedBox(width: 8),
                                  Container(
                                    padding: const EdgeInsets.symmetric(
                                      horizontal: 6,
                                      vertical: 2,
                                    ),
                                    decoration: BoxDecoration(
                                      color: Theme.of(context)
                                          .colorScheme
                                          .secondaryContainer,
                                      borderRadius: BorderRadius.circular(4),
                                    ),
                                    child: Text(
                                      node.nodeType,
                                      style: TextStyle(
                                        fontSize: 10,
                                        color: Theme.of(context)
                                            .colorScheme
                                            .onSecondaryContainer,
                                      ),
                                    ),
                                  ),
                                  const Spacer(),
                                  Text(
                                    '${node.durationMs} ms',
                                    style: TextStyle(
                                      fontSize: 11,
                                      color:
                                          Theme.of(context).colorScheme.outline,
                                    ),
                                  ),
                                ],
                              ),
                              if (node.outputSummary.isNotEmpty) ...[
                                const SizedBox(height: 4),
                                Text(
                                  node.outputSummary,
                                  style: Theme.of(context).textTheme.bodySmall,
                                ),
                              ],
                            ],
                          ),
                        ),
                      ],
                    ),
                  );
                }).toList(),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
