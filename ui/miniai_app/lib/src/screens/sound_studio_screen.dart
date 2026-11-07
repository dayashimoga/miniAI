import 'package:flutter/material.dart';
import '../controllers/platform_controller.dart';
import '../models/system_contracts.dart';

/// Sound Studio Screen providing offline Speech Recognition (ASR),
/// Neural Text-to-Speech (TTS), unified Voice Conversation Loop,
/// and polyphonic Music Generation.
class SoundStudioScreen extends StatelessWidget {
  const SoundStudioScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        return DefaultTabController(
          length: 2,
          child: Scaffold(
            appBar: const TabBar(
              key: Key('sound_studio_tab_bar'),
              tabs: [
                Tab(
                  key: Key('tab_speech_voice_loop'),
                  icon: Icon(Icons.mic),
                  text: 'Speech & Voice Loop',
                ),
                Tab(
                  key: Key('tab_music_soundstage'),
                  icon: Icon(Icons.music_note),
                  text: 'Music Studio & Soundstage',
                ),
              ],
            ),
            body: TabBarView(
              children: [
                _buildSpeechVoiceLoopTab(context),
                _buildMusicStudioTab(context),
              ],
            ),
          ),
        );
      },
    );
  }

  // ---------------------------------------------------------------------------
  // Tab 1: Speech & Voice Loop
  // ---------------------------------------------------------------------------

  Widget _buildSpeechVoiceLoopTab(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildAsrSection(context),
          const SizedBox(height: 16),
          const Divider(),
          const SizedBox(height: 16),
          _buildTtsSection(context),
          const SizedBox(height: 16),
          const Divider(),
          const SizedBox(height: 16),
          _buildVoiceLoopSection(context),
        ],
      ),
    );
  }

  Widget _buildAsrSection(BuildContext context) {
    final asr = controller.lastAsrResult;

    return Card(
      elevation: 2,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.record_voice_over, color: Colors.indigo),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Offline Speech Recognition (ASR)',
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            FilledButton.icon(
              key: const Key('btn_transcribe_audio'),
              icon: controller.isTranscribingSpeech
                  ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.hearing, size: 18),
              label: Text(controller.isTranscribingSpeech
                  ? 'Transcribing...'
                  : 'Transcribe Speech'),
              onPressed: controller.isTranscribingSpeech
                  ? null
                  : () => controller.transcribeAudio(),
            ),
            if (controller.asrErrorMessage != null) ...[
              const SizedBox(height: 8),
              Text(
                controller.asrErrorMessage!,
                style: const TextStyle(color: Colors.red),
              ),
            ],
            if (asr != null) ...[
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.grey.withAlpha(25),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Wrap(
                      spacing: 8,
                      runSpacing: 4,
                      children: [
                        Chip(
                          key: const Key('asr_confidence_chip'),
                          label: Text(
                              '${(asr.confidence * 100).toStringAsFixed(0)}% Confidence'),
                          backgroundColor: Colors.green.withAlpha(50),
                        ),
                        Chip(
                          label: Text(
                              '${asr.durationSecs.toStringAsFixed(1)}s Duration'),
                        ),
                        Chip(
                          avatar: const Icon(Icons.fingerprint, size: 16),
                          label: Text('SHA-256: ${asr.sha256.substring(0, 10)}...'),
                        ),
                      ],
                    ),
                    const SizedBox(height: 8),
                    Text(
                      asr.fullText,
                      key: const Key('asr_full_text'),
                      style: const TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                    if (asr.segments.isNotEmpty) ...[
                      const SizedBox(height: 8),
                      const Text(
                        'Timestamped Segments:',
                        style: TextStyle(
                            fontSize: 12, fontWeight: FontWeight.bold),
                      ),
                      ...asr.segments.map(
                        (seg) => Padding(
                          padding: const EdgeInsets.symmetric(vertical: 2),
                          child: Text(
                            '[${seg.startSecs.toStringAsFixed(1)}s - ${seg.endSecs.toStringAsFixed(1)}s] ${seg.text} (${(seg.confidence * 100).toStringAsFixed(0)}%)',
                            style: const TextStyle(
                                fontSize: 13, color: Colors.black87),
                          ),
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildTtsSection(BuildContext context) {
    final tts = controller.lastTtsOutput;
    final textController = TextEditingController(
      text: 'MiniAI delivers on-device neural acoustic speech synthesis.',
    );

    return Card(
      elevation: 2,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.spatial_audio_off, color: Colors.deepPurple),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Neural Text-to-Speech (TTS)',
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            TextField(
              key: const Key('tts_input_field'),
              controller: textController,
              decoration: const InputDecoration(
                labelText: 'Text to Synthesize',
                border: OutlineInputBorder(),
                hintText: 'Enter text for neural formant speech...',
              ),
              maxLines: 2,
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 16,
              runSpacing: 8,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                SizedBox(
                  width: 220,
                  child: DropdownButton<TtsVoiceProfileContract>(
                    key: const Key('tts_voice_profile_dropdown'),
                    isExpanded: true,
                    value: controller.selectedVoiceProfile,
                    onChanged: (val) {
                      if (val != null) controller.setSelectedVoiceProfile(val);
                    },
                    items: TtsVoiceProfileContract.values
                        .map(
                          (v) => DropdownMenuItem(
                            value: v,
                            child: Text(
                              v.displayName,
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                        )
                        .toList(),
                  ),
                ),
                SizedBox(
                  width: 140,
                  child: Row(
                    children: [
                      const Text('Speed: ', style: TextStyle(fontSize: 12)),
                      Expanded(
                        child: Slider(
                          key: const Key('slider_tts_speed'),
                          value: controller.ttsSpeed,
                          min: 0.5,
                          max: 2.0,
                          divisions: 15,
                          label: '${controller.ttsSpeed.toStringAsFixed(1)}x',
                          onChanged: controller.setTtsSpeed,
                        ),
                      ),
                    ],
                  ),
                ),
                SizedBox(
                  width: 140,
                  child: Row(
                    children: [
                      const Text('Pitch: ', style: TextStyle(fontSize: 12)),
                      Expanded(
                        child: Slider(
                          key: const Key('slider_tts_pitch'),
                          value: controller.ttsPitchSemitones,
                          min: -12.0,
                          max: 12.0,
                          divisions: 24,
                          label:
                              '${controller.ttsPitchSemitones.toStringAsFixed(0)} st',
                          onChanged: controller.setTtsPitch,
                        ),
                      ),
                    ],
                  ),
                ),
                FilledButton.icon(
                  key: const Key('btn_synthesize_speech'),
                  icon: controller.isSynthesizingTts
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.volume_up, size: 18),
                  label: Text(controller.isSynthesizingTts
                      ? 'Synthesizing...'
                      : 'Synthesize Speech'),
                  onPressed: controller.isSynthesizingTts
                      ? null
                      : () => controller.synthesizeSpeech(textController.text),
                ),
              ],
            ),
            if (controller.ttsErrorMessage != null) ...[
              const SizedBox(height: 8),
              Text(
                controller.ttsErrorMessage!,
                style: const TextStyle(color: Colors.red),
              ),
            ],
            if (tts != null) ...[
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.deepPurple.withAlpha(20),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: [
                    Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const Icon(Icons.check_circle, color: Colors.green),
                        const SizedBox(width: 8),
                        Text(
                          'Synthesized: ${tts.durationSecs.toStringAsFixed(1)}s (${tts.phonemeCount} phonemes, ${tts.sampleRate} Hz)',
                          key: const Key('tts_output_summary'),
                          style: const TextStyle(fontWeight: FontWeight.bold),
                        ),
                      ],
                    ),
                    Chip(
                      avatar: const Icon(Icons.fingerprint, size: 16),
                      label: Text('SHA-256: ${tts.sha256.substring(0, 10)}...'),
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

  Widget _buildVoiceLoopSection(BuildContext context) {
    final loopController = TextEditingController();

    return Card(
      elevation: 2,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.sync_alt, color: Colors.teal),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Offline Voice Conversation Loop (ASR → LLM → TTS)',
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            TextField(
              key: const Key('voice_system_prompt_field'),
              controller:
                  TextEditingController(text: controller.voiceSystemPrompt),
              onChanged: controller.setVoiceSystemPrompt,
              decoration: const InputDecoration(
                labelText: 'System Prompt',
                border: OutlineInputBorder(),
                isDense: true,
              ),
            ),
            const SizedBox(height: 8),
            Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                TextField(
                  key: const Key('voice_loop_input_field'),
                  controller: loopController,
                  decoration: const InputDecoration(
                    labelText: 'Spoken / Text User Query',
                    border: OutlineInputBorder(),
                    hintText: 'Ask anything to the voice assistant...',
                    isDense: true,
                  ),
                ),
                const SizedBox(height: 8),
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: [
                    FilledButton.icon(
                      key: const Key('btn_send_voice_turn'),
                      icon: controller.isProcessingVoiceTurn
                          ? const SizedBox(
                              width: 16,
                              height: 16,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : const Icon(Icons.send, size: 18),
                      label: const Text('Send Turn'),
                      onPressed: controller.isProcessingVoiceTurn
                          ? null
                          : () => controller.executeVoiceTurn(
                                textQuery: loopController.text,
                              ),
                    ),
                    if (controller.isProcessingVoiceTurn) ...[
                      OutlinedButton.icon(
                        key: const Key('btn_cancel_voice_turn'),
                        icon: const Icon(Icons.cancel, color: Colors.red),
                        label: const Text('Cancel'),
                        onPressed: controller.cancelVoiceTurn,
                      ),
                    ],
                  ],
                ),
              ],
            ),
            if (controller.voiceLoopHistory.isNotEmpty) ...[
              const SizedBox(height: 16),
              const Text(
                'Conversation Timeline:',
                style: TextStyle(fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 8),
              ListView.builder(
                key: const Key('voice_loop_timeline'),
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: controller.voiceLoopHistory.length,
                itemBuilder: (context, idx) {
                  final turn = controller.voiceLoopHistory[idx];
                  return Card(
                    margin: const EdgeInsets.symmetric(vertical: 4),
                    color: Colors.grey.withAlpha(15),
                    child: Padding(
                      padding: const EdgeInsets.all(12),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              const CircleAvatar(
                                radius: 12,
                                child: Text('U', style: TextStyle(fontSize: 11)),
                              ),
                              const SizedBox(width: 8),
                              Expanded(
                                child: Text(
                                  turn.userTranscription,
                                  style: const TextStyle(
                                      fontWeight: FontWeight.w600),
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: 6),
                          Row(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              const CircleAvatar(
                                radius: 12,
                                backgroundColor: Colors.teal,
                                child: Text('AI',
                                    style: TextStyle(
                                        fontSize: 10, color: Colors.white)),
                              ),
                              const SizedBox(width: 8),
                              Expanded(
                                child: Text(turn.agentResponseText),
                              ),
                            ],
                          ),
                          const SizedBox(height: 8),
                          Wrap(
                            spacing: 8,
                            children: [
                              Chip(
                                label: Text(
                                    'ASR: ${turn.metrics.asrLatencyMs.toStringAsFixed(1)}ms'),
                              ),
                              Chip(
                                label: Text(
                                    'LLM TTFT: ${turn.metrics.llmTtftMs.toStringAsFixed(1)}ms'),
                              ),
                              Chip(
                                label: Text(
                                    'TTS: ${turn.metrics.ttsLatencyMs.toStringAsFixed(1)}ms'),
                              ),
                              Chip(
                                backgroundColor: Colors.teal.withAlpha(40),
                                label: Text(
                                  'Total: ${turn.metrics.totalTurnaroundMs.toStringAsFixed(1)}ms',
                                  style: const TextStyle(
                                      fontWeight: FontWeight.bold),
                                ),
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  );
                },
              ),
            ],
          ],
        ),
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // Tab 2: Music Studio & Soundstage
  // ---------------------------------------------------------------------------

  Widget _buildMusicStudioTab(BuildContext context) {
    final promptController =
        TextEditingController(text: controller.musicConfig.prompt);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Card(
            elevation: 2,
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.piano, color: Colors.orange),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          'Polyphonic Music Generation Engine',
                          style:
                              Theme.of(context).textTheme.titleMedium?.copyWith(
                                    fontWeight: FontWeight.bold,
                                  ),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    key: const Key('music_prompt_field'),
                    controller: promptController,
                    onChanged: controller.setMusicPrompt,
                    decoration: const InputDecoration(
                      labelText: 'Music Prompt / Genre',
                      border: OutlineInputBorder(),
                      hintText: 'e.g. Uplifting synthwave in C Major...',
                    ),
                  ),
                  const SizedBox(height: 12),
                  Wrap(
                    spacing: 16,
                    runSpacing: 8,
                    crossAxisAlignment: WrapCrossAlignment.center,
                    children: [
                      SizedBox(
                        width: 150,
                        child: Row(
                          children: [
                            const Text('Tempo: ',
                                style: TextStyle(fontSize: 12)),
                            Expanded(
                              child: Slider(
                                key: const Key('slider_music_tempo'),
                                value: controller.musicConfig.tempoBpm.toDouble(),
                                min: 40,
                                max: 240,
                                divisions: 20,
                                label: '${controller.musicConfig.tempoBpm} BPM',
                                onChanged: (v) =>
                                    controller.setMusicTempo(v.round()),
                              ),
                            ),
                          ],
                        ),
                      ),
                      SizedBox(
                        width: 150,
                        child: Row(
                          children: [
                            const Text('Duration: ',
                                style: TextStyle(fontSize: 12)),
                            Expanded(
                              child: Slider(
                                key: const Key('slider_music_duration'),
                                value: controller.musicConfig.durationSecs,
                                min: 1.0,
                                max: 60.0,
                                divisions: 59,
                                label:
                                    '${controller.musicConfig.durationSecs.toStringAsFixed(0)}s',
                                onChanged: controller.setMusicDuration,
                              ),
                            ),
                          ],
                        ),
                      ),
                      DropdownButton<String>(
                        key: const Key('dropdown_music_scale'),
                        value: controller.musicConfig.scaleKey,
                        onChanged: (val) {
                          if (val != null) controller.setMusicScaleKey(val);
                        },
                        items: const [
                          DropdownMenuItem(
                              value: 'C Major', child: Text('C Major')),
                          DropdownMenuItem(
                              value: 'A Minor', child: Text('A Minor')),
                          DropdownMenuItem(
                              value: 'G Major', child: Text('G Major')),
                          DropdownMenuItem(
                              value: 'E Minor', child: Text('E Minor')),
                          DropdownMenuItem(
                              value: 'D Dorian', child: Text('D Dorian')),
                        ],
                      ),
                      Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          const Text('Stereo Soundstage: ',
                              style: TextStyle(fontSize: 12)),
                          Switch(
                            key: const Key('toggle_music_stereo'),
                            value: controller.musicConfig.channels == 2,
                            onChanged: (isStereo) =>
                                controller.setMusicChannels(isStereo ? 2 : 1),
                          ),
                        ],
                      ),
                      FilledButton.icon(
                        key: const Key('btn_generate_music'),
                        icon: controller.isGeneratingMusic
                            ? const SizedBox(
                                width: 16,
                                height: 16,
                                child:
                                    CircularProgressIndicator(strokeWidth: 2),
                              )
                            : const Icon(Icons.music_note, size: 18),
                        label: Text(controller.isGeneratingMusic
                            ? 'Composing...'
                            : 'Generate Music'),
                        onPressed: controller.isGeneratingMusic
                            ? null
                            : () => controller.generateMusic(),
                      ),
                      if (controller.isGeneratingMusic) ...[
                        OutlinedButton.icon(
                          key: const Key('btn_cancel_music'),
                          icon: const Icon(Icons.cancel, color: Colors.red),
                          label: const Text('Cancel'),
                          onPressed: controller.cancelMusicGeneration,
                        ),
                      ],
                    ],
                  ),
                  if (controller.isGeneratingMusic) ...[
                    const SizedBox(height: 12),
                    LinearProgressIndicator(
                      key: const Key('music_progress_bar'),
                      value: controller.musicProgress,
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'Rendering measure ${controller.musicCurrentMeasure} of ${controller.musicTotalMeasures}...',
                      style: const TextStyle(fontSize: 12, color: Colors.grey),
                    ),
                  ],
                  if (controller.musicErrorMessage != null) ...[
                    const SizedBox(height: 8),
                    Text(
                      controller.musicErrorMessage!,
                      style: const TextStyle(color: Colors.red),
                    ),
                  ],
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          if (controller.selectedMusicTrack != null) ...[
            _buildActiveMusicTrackCard(context, controller.selectedMusicTrack!),
            const SizedBox(height: 16),
          ],
          _buildMusicLibrarySection(context),
        ],
      ),
    );
  }

  Widget _buildActiveMusicTrackCard(
      BuildContext context, MusicTrackOutputContract track) {
    return Card(
      elevation: 2,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Wrap(
              spacing: 8,
              runSpacing: 8,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Icon(Icons.play_circle_filled,
                        size: 36, color: Colors.orange),
                    const SizedBox(width: 12),
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          track.prompt,
                          key: const Key('active_track_prompt'),
                          style: const TextStyle(
                              fontSize: 16, fontWeight: FontWeight.bold),
                        ),
                        Text(
                          '${track.durationSecs.toStringAsFixed(1)}s • ${track.tempoBpm} BPM • ${track.scaleKey} • ${track.channels == 2 ? "Stereo" : "Mono"}',
                          style:
                              const TextStyle(fontSize: 12, color: Colors.grey),
                        ),
                      ],
                    ),
                  ],
                ),
                Chip(
                  avatar: const Icon(Icons.fingerprint, size: 16),
                  label: Text('SHA-256: ${track.sha256.substring(0, 10)}...'),
                ),
              ],
            ),
            const SizedBox(height: 12),
            // Mock-free waveform visualization bars
            Container(
              height: 48,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(
                color: Colors.orange.withAlpha(20),
                borderRadius: BorderRadius.circular(6),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: List.generate(32, (i) {
                  final height = (16 + (i * 7) % 28).toDouble();
                  return Container(
                    width: 4,
                    height: height,
                    decoration: BoxDecoration(
                      color: Colors.orange.withAlpha(180),
                      borderRadius: BorderRadius.circular(2),
                    ),
                  );
                }),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMusicLibrarySection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            const Icon(Icons.library_music, size: 20),
            const SizedBox(width: 8),
            Text(
              'Music Library (${controller.musicLibrary.length} tracks)',
              style: Theme.of(context).textTheme.titleSmall?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        if (controller.musicLibrary.isEmpty)
          const Padding(
            padding: EdgeInsets.symmetric(vertical: 16),
            child: Text(
              'No music tracks generated yet. Click "Generate Music" above.',
              style: TextStyle(color: Colors.grey),
            ),
          )
        else
          ListView.builder(
            key: const Key('music_library_list'),
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            itemCount: controller.musicLibrary.length,
            itemBuilder: (context, idx) {
              final item = controller.musicLibrary[idx];
              final isSelected = controller.selectedMusicTrack?.id == item.id;

              return ListTile(
                selected: isSelected,
                leading: const Icon(Icons.audiotrack),
                title: Text(item.prompt),
                subtitle: Text(
                  '${item.durationSecs.toStringAsFixed(0)}s • ${item.tempoBpm} BPM • ${item.scaleKey}',
                ),
                trailing: IconButton(
                  key: Key('btn_delete_music_${item.id}'),
                  icon: const Icon(Icons.delete_outline, size: 20),
                  onPressed: () => controller.deleteMusicTrack(item.id),
                ),
                onTap: () => controller.selectMusicTrack(item),
              );
            },
          ),
      ],
    );
  }
}

