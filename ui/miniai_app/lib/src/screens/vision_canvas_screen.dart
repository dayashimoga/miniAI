import 'dart:math';
import 'package:flutter/material.dart';
import '../controllers/platform_controller.dart';
import '../models/system_contracts.dart';

/// Screen providing Vision Understanding (OCR, captioning, grounded VQA)
/// and Canvas Studio (Text-to-Image, Image-to-Image, Inpainting mask canvas,
/// and local gallery with cryptographic provenance inspection).
class VisionCanvasScreen extends StatefulWidget {
  const VisionCanvasScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  State<VisionCanvasScreen> createState() => _VisionCanvasScreenState();
}

class _VisionCanvasScreenState extends State<VisionCanvasScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final TextEditingController _qaController = TextEditingController();
  final TextEditingController _promptController = TextEditingController();
  final TextEditingController _negPromptController = TextEditingController();
  final TextEditingController _seedController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    _promptController.text = widget.controller.imageGenConfig.prompt;
    _negPromptController.text = widget.controller.imageGenConfig.negativePrompt;
    _seedController.text = widget.controller.imageGenConfig.seed.toString();
  }

  @override
  void dispose() {
    _tabController.dispose();
    _qaController.dispose();
    _promptController.dispose();
    _negPromptController.dispose();
    _seedController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        return Scaffold(
          appBar: PreferredSize(
            preferredSize: const Size.fromHeight(48),
            child: Container(
              color: Theme.of(context).colorScheme.surface,
              child: TabBar(
                controller: _tabController,
                tabs: const [
                  Tab(
                    key: Key('tab_vision_understanding'),
                    icon: Icon(Icons.remove_red_eye_outlined),
                    text: 'Vision Understanding',
                  ),
                  Tab(
                    key: Key('tab_canvas_studio'),
                    icon: Icon(Icons.palette_outlined),
                    text: 'Canvas Studio & Gallery',
                  ),
                ],
              ),
            ),
          ),
          body: TabBarView(
            controller: _tabController,
            children: [
              _buildVisionUnderstandingView(),
              _buildCanvasStudioView(),
            ],
          ),
        );
      },
    );
  }

  // =========================================================================
  // Vision Understanding Tab
  // =========================================================================

  Widget _buildVisionUnderstandingView() {
    final controller = widget.controller;
    final result = controller.visionAnalysisResult;

    return SingleChildScrollView(
      key: const Key('vision_scroll_view'),
      padding: const EdgeInsets.all(16.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Upload & Sample Bar
          Card(
            key: const Key('vision_upload_card'),
            elevation: 1,
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.cloud_upload_outlined, color: Colors.blue),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          'Local Vision Inspection',
                          style: Theme.of(context).textTheme.titleMedium?.copyWith(
                                fontWeight: FontWeight.bold,
                              ),
                        ),
                      ),
                      const Chip(
                        key: Key('vision_cpu_badge'),
                        avatar: Icon(Icons.memory, size: 16),
                        label: Text('CPU-First'),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Analyze local images securely under Local-Only Privacy Mode. Evaluates magic bytes, formats (PNG, JPEG, WebP), dimensions, and runs OCR detection.',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                  const SizedBox(height: 16),
                  Wrap(
                    spacing: 12,
                    runSpacing: 8,
                    children: [
                      ElevatedButton.icon(
                        key: const Key('vision_upload_button'),
                        icon: const Icon(Icons.upload_file),
                        label: const Text('Simulate Image Upload'),
                        onPressed: () {
                          controller.loadSampleVisionImage();
                        },
                      ),
                      OutlinedButton.icon(
                        key: const Key('load_sample_image_button'),
                        icon: const Icon(Icons.auto_awesome),
                        label: const Text('Load Sample Diagram'),
                        onPressed: () {
                          controller.loadSampleVisionImage();
                        },
                      ),
                    ],
                  ),
                  if (controller.visionErrorMessage != null) ...[
                    const SizedBox(height: 12),
                    Container(
                      key: const Key('vision_error_banner'),
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: Colors.red.withAlpha(25),
                        border: Border.all(color: Colors.red.withAlpha(100)),
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Row(
                        children: [
                          const Icon(Icons.error_outline, color: Colors.red),
                          const SizedBox(width: 8),
                          Expanded(
                            child: Text(
                              controller.visionErrorMessage!,
                              style: const TextStyle(color: Colors.red),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),

          // Inspection Viewer
          if (result != null) ...[
            Card(
              key: const Key('vision_image_viewer'),
              elevation: 1,
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        const Icon(Icons.image_outlined, color: Colors.purple),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            'Active Image Metadata',
                            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                                  fontWeight: FontWeight.bold,
                                ),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 12),
                    Wrap(
                      spacing: 8,
                      runSpacing: 8,
                      children: [
                        Chip(
                          key: const Key('vision_format_badge'),
                          label: Text('Format: ${result.format}'),
                        ),
                        Chip(
                          key: const Key('vision_dimensions_badge'),
                          label: Text('${result.width} x ${result.height} px'),
                        ),
                        Chip(
                          key: const Key('vision_sha256_badge'),
                          avatar: const Icon(Icons.fingerprint, size: 16),
                          label: Text(
                            'SHA: ${result.imageSha256.length > 12 ? "${result.imageSha256.substring(0, 8)}..." : result.imageSha256}',
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),

            // OCR Detections
            Card(
              key: const Key('ocr_result_card'),
              elevation: 1,
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        const Icon(Icons.text_snippet_outlined, color: Colors.orange),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            'OCR Bounding Box Detections (${result.ocrBoxes.length})',
                            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                                  fontWeight: FontWeight.bold,
                                ),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 12),
                    ListView.separated(
                      shrinkWrap: true,
                      physics: const NeverScrollableScrollPhysics(),
                      itemCount: result.ocrBoxes.length,
                      separatorBuilder: (_, __) => const Divider(height: 8),
                      itemBuilder: (context, idx) {
                        final box = result.ocrBoxes[idx];
                        return ListTile(
                          key: Key('ocr_box_$idx'),
                          dense: true,
                          leading: CircleAvatar(
                            radius: 12,
                            child: Text('${idx + 1}', style: const TextStyle(fontSize: 11)),
                          ),
                          title: Text(box.text, style: const TextStyle(fontWeight: FontWeight.bold)),
                          subtitle: Text(
                            'Bounds: [x: ${box.x.toStringAsFixed(2)}, y: ${box.y.toStringAsFixed(2)}, w: ${box.width.toStringAsFixed(2)}, h: ${box.height.toStringAsFixed(2)}]',
                          ),
                          trailing: Chip(
                            key: Key('ocr_confidence_chip_$idx'),
                            label: Text('${(box.confidence * 100).toInt()}%'),
                          ),
                        );
                      },
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Scene Description & Tags
            Card(
              key: const Key('vision_description_card'),
              elevation: 1,
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        const Icon(Icons.description_outlined, color: Colors.teal),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            'Visual Scene Understanding',
                            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                                  fontWeight: FontWeight.bold,
                                ),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 12),
                    Text(result.description, style: Theme.of(context).textTheme.bodyMedium),
                    const SizedBox(height: 12),
                    Wrap(
                      spacing: 8,
                      runSpacing: 6,
                      children: result.tags.map((tag) {
                        return Chip(
                          key: Key('vision_tag_chip_$tag'),
                          label: Text('#$tag'),
                          backgroundColor: Colors.teal.withAlpha(25),
                        );
                      }).toList(),
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Visual Question Answering
            Card(
              key: const Key('vision_qa_card'),
              elevation: 1,
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        const Icon(Icons.question_answer_outlined, color: Colors.deepPurple),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            'Grounded Visual Q&A',
                            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                                  fontWeight: FontWeight.bold,
                                ),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 12),
                    Row(
                      children: [
                        Expanded(
                          child: TextField(
                            key: const Key('vision_qa_field'),
                            controller: _qaController,
                            decoration: const InputDecoration(
                              hintText: 'Ask a question about the image (e.g. "What text is shown?")...',
                              border: OutlineInputBorder(),
                              isDense: true,
                            ),
                            onSubmitted: (q) {
                              if (q.trim().isNotEmpty) {
                                controller.askVisionQuestion(q);
                              }
                            },
                          ),
                        ),
                        const SizedBox(width: 12),
                        ElevatedButton.icon(
                          key: const Key('vision_qa_submit_button'),
                          icon: const Icon(Icons.send),
                          label: const Text('Ask'),
                          onPressed: () {
                            if (_qaController.text.trim().isNotEmpty) {
                              controller.askVisionQuestion(_qaController.text);
                            }
                          },
                        ),
                      ],
                    ),
                    if (result.qaAnswer != null) ...[
                      const SizedBox(height: 16),
                      Container(
                        key: const Key('vision_qa_answer_card'),
                        padding: const EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: Colors.deepPurple.withAlpha(20),
                          borderRadius: BorderRadius.circular(8),
                          border: Border.all(color: Colors.deepPurple.withAlpha(80)),
                        ),
                        child: Row(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            const Icon(Icons.lightbulb_outline, color: Colors.deepPurple),
                            const SizedBox(width: 12),
                            Expanded(
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  const Text(
                                    'Grounded Answer',
                                    style: TextStyle(fontWeight: FontWeight.bold),
                                  ),
                                  const SizedBox(height: 4),
                                  Text(
                                    result.qaAnswer!,
                                    key: const Key('vision_qa_answer_text'),
                                  ),
                                ],
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }

  // =========================================================================
  // Canvas Studio & Generator Tab
  // =========================================================================

  Widget _buildCanvasStudioView() {
    final controller = widget.controller;

    return LayoutBuilder(
      builder: (context, constraints) {
        final isWide = constraints.maxWidth >= 900;

        if (isWide) {
          return Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                flex: 5,
                child: SingleChildScrollView(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      _buildGenerationControlsCard(),
                      const SizedBox(height: 16),
                      _buildInpaintingCanvasCard(),
                      const SizedBox(height: 16),
                      if (controller.isGeneratingImage) ...[
                        _buildGenerationProgressCard(),
                        const SizedBox(height: 16),
                      ],
                      _buildGenerateActionBar(),
                    ],
                  ),
                ),
              ),
              const VerticalDivider(width: 1, thickness: 1),
              Expanded(
                flex: 4,
                child: SingleChildScrollView(
                  padding: const EdgeInsets.all(16.0),
                  child: _buildGallerySection(),
                ),
              ),
            ],
          );
        }

        return SingleChildScrollView(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _buildGenerationControlsCard(),
              const SizedBox(height: 16),
              _buildInpaintingCanvasCard(),
              const SizedBox(height: 16),
              if (controller.isGeneratingImage) ...[
                _buildGenerationProgressCard(),
                const SizedBox(height: 16),
              ],
              _buildGenerateActionBar(),
              const SizedBox(height: 24),
              const Divider(thickness: 1),
              const SizedBox(height: 16),
              _buildGallerySection(),
            ],
          ),
        );
      },
    );
  }

  Widget _buildGenerationControlsCard() {
    final controller = widget.controller;
    final config = controller.imageGenConfig;

    return Card(
      key: const Key('imagegen_controls_card'),
      elevation: 1,
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.tune, color: Colors.indigo),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Generation Parameters',
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),

            // Mode Selector
            SegmentedButton<GenerationModeContract>(
              key: const Key('imagegen_mode_selector'),
              segments: const [
                ButtonSegment(
                  value: GenerationModeContract.textToImage,
                  label: Text('Text-to-Image'),
                  icon: Icon(Icons.text_fields),
                ),
                ButtonSegment(
                  value: GenerationModeContract.imageToImage,
                  label: Text('Img-to-Img'),
                  icon: Icon(Icons.image),
                ),
                ButtonSegment(
                  value: GenerationModeContract.inpainting,
                  label: Text('Inpainting'),
                  icon: Icon(Icons.brush),
                ),
              ],
              selected: {config.mode},
              onSelectionChanged: (modes) {
                controller.setImageGenMode(modes.first);
              },
            ),
            const SizedBox(height: 16),

            // Prompt Field
            TextField(
              key: const Key('imagegen_prompt_field'),
              controller: _promptController,
              decoration: const InputDecoration(
                labelText: 'Synthesis Prompt',
                hintText: 'e.g. A serene mountain lake at sunrise...',
                border: OutlineInputBorder(),
                isDense: true,
              ),
              maxLines: 2,
              onChanged: (val) => controller.setImageGenPrompt(val),
            ),
            const SizedBox(height: 12),

            // Negative Prompt Field
            TextField(
              key: const Key('imagegen_negative_prompt_field'),
              controller: _negPromptController,
              decoration: const InputDecoration(
                labelText: 'Negative Prompt',
                hintText: 'blurry, oversaturated, low quality...',
                border: OutlineInputBorder(),
                isDense: true,
              ),
              onChanged: (val) => controller.setImageGenNegativePrompt(val),
            ),
            const SizedBox(height: 16),

            // Dimensions Picker
            Row(
              children: [
                const Text('Dimensions:', style: TextStyle(fontWeight: FontWeight.bold)),
                const SizedBox(width: 12),
                ChoiceChip(
                  key: const Key('dimension_chip_256'),
                  label: const Text('256 x 256'),
                  selected: config.width == 256 && config.height == 256,
                  onSelected: (sel) {
                    if (sel) controller.setImageGenDimensions(256, 256);
                  },
                ),
                const SizedBox(width: 8),
                ChoiceChip(
                  key: const Key('dimension_chip_512'),
                  label: const Text('512 x 512'),
                  selected: config.width == 512 && config.height == 512,
                  onSelected: (sel) {
                    if (sel) controller.setImageGenDimensions(512, 512);
                  },
                ),
              ],
            ),
            const SizedBox(height: 12),

            // Denoising Steps Slider
            Row(
              children: [
                Text(
                  'Denoising Steps: ${config.steps}',
                  key: const Key('imagegen_steps_value'),
                  style: const TextStyle(fontWeight: FontWeight.bold),
                ),
                Expanded(
                  child: Slider(
                    key: const Key('imagegen_steps_slider'),
                    value: config.steps.toDouble(),
                    min: 5,
                    max: 50,
                    divisions: 45,
                    onChanged: (val) => controller.setImageGenSteps(val.toInt()),
                  ),
                ),
              ],
            ),

            // Guidance Scale Slider
            Row(
              children: [
                Text(
                  'Guidance: ${config.guidanceScale.toStringAsFixed(1)}',
                  key: const Key('imagegen_guidance_value'),
                  style: const TextStyle(fontWeight: FontWeight.bold),
                ),
                Expanded(
                  child: Slider(
                    key: const Key('imagegen_guidance_slider'),
                    value: config.guidanceScale,
                    min: 1.0,
                    max: 15.0,
                    divisions: 28,
                    onChanged: (val) => controller.setImageGenGuidance(val),
                  ),
                ),
              ],
            ),

            // Seed & Randomizer
            Row(
              children: [
                Expanded(
                  child: TextField(
                    key: const Key('imagegen_seed_field'),
                    controller: _seedController,
                    decoration: const InputDecoration(
                      labelText: 'Deterministic Seed',
                      border: OutlineInputBorder(),
                      isDense: true,
                    ),
                    keyboardType: TextInputType.number,
                    onChanged: (val) {
                      final parsed = int.tryParse(val);
                      if (parsed != null) controller.setImageGenSeed(parsed);
                    },
                  ),
                ),
                const SizedBox(width: 12),
                OutlinedButton.icon(
                  key: const Key('imagegen_random_seed_button'),
                  icon: const Icon(Icons.casino),
                  label: const Text('Random'),
                  onPressed: () {
                    final rand = Random().nextInt(1000000);
                    _seedController.text = rand.toString();
                    controller.setImageGenSeed(rand);
                  },
                ),
              ],
            ),

            // Strength (for img2img / inpainting)
            if (config.mode != GenerationModeContract.textToImage) ...[
              const SizedBox(height: 12),
              Row(
                children: [
                  Text(
                    'Strength: ${config.strength.toStringAsFixed(2)}',
                    key: const Key('imagegen_strength_value'),
                    style: const TextStyle(fontWeight: FontWeight.bold),
                  ),
                  Expanded(
                    child: Slider(
                      key: const Key('imagegen_strength_slider'),
                      value: config.strength,
                      min: 0.1,
                      max: 1.0,
                      divisions: 18,
                      onChanged: (val) => controller.setImageGenStrength(val),
                    ),
                  ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildInpaintingCanvasCard() {
    final controller = widget.controller;

    return Card(
      key: const Key('inpainting_canvas_card'),
      elevation: 1,
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.brush, color: Colors.teal),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Inpainting Mask Canvas',
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                  ),
                ),
                Chip(
                  key: const Key('mask_points_chip'),
                  label: Text('${controller.maskPoints.length} Points'),
                ),
              ],
            ),
            const SizedBox(height: 12),
            // Brush Toolbar
            Wrap(
              spacing: 8,
              runSpacing: 8,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                Text(
                  'Brush: ${controller.brushSize.toInt()}px',
                  key: const Key('brush_size_value'),
                  style: const TextStyle(fontWeight: FontWeight.bold),
                ),
                SizedBox(
                  width: 140,
                  child: Slider(
                    key: const Key('brush_size_slider'),
                    value: controller.brushSize,
                    min: 8.0,
                    max: 48.0,
                    divisions: 10,
                    onChanged: (val) => controller.setBrushSize(val),
                  ),
                ),
                ChoiceChip(
                  key: const Key('eraser_toggle_button'),
                  avatar: Icon(
                    controller.isEraserMode ? Icons.cleaning_services : Icons.brush,
                    size: 16,
                  ),
                  label: Text(controller.isEraserMode ? 'Eraser' : 'Draw'),
                  selected: controller.isEraserMode,
                  onSelected: (val) => controller.setEraserMode(val),
                ),
                OutlinedButton.icon(
                  key: const Key('clear_mask_button'),
                  icon: const Icon(Icons.delete_outline, size: 16),
                  label: const Text('Clear'),
                  onPressed: () => controller.clearMask(),
                ),
              ],
            ),
            const SizedBox(height: 12),

            // Canvas Area
            Container(
              height: 240,
              width: double.infinity,
              decoration: BoxDecoration(
                color: const Color(0xFF1E1E2E),
                border: Border.all(color: Colors.grey.withAlpha(100)),
                borderRadius: BorderRadius.circular(8),
              ),
              child: ClipRRect(
                borderRadius: BorderRadius.circular(8),
                child: GestureDetector(
                  key: const Key('inpainting_mask_canvas'),
                  onPanDown: (details) {
                    controller.addMaskPoint(details.localPosition);
                  },
                  onPanUpdate: (details) {
                    controller.addMaskPoint(details.localPosition);
                  },
                  child: CustomPaint(
                    size: Size.infinite,
                    painter: _MaskCanvasPainter(
                      points: controller.maskPoints,
                      brushSize: controller.brushSize,
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildGenerationProgressCard() {
    final controller = widget.controller;

    return Card(
      key: const Key('imagegen_progress_card'),
      color: Colors.blue.withAlpha(25),
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
                const SizedBox(width: 12),
                const Expanded(
                  child: Text(
                    'Synthesizing Image on CPU...',
                    style: TextStyle(fontWeight: FontWeight.bold),
                  ),
                ),
                Chip(
                  key: const Key('imagegen_loss_badge'),
                  label: Text('Loss: ${controller.imageGenStepLoss.toStringAsFixed(3)}'),
                ),
              ],
            ),
            const SizedBox(height: 12),
            LinearProgressIndicator(
              key: const Key('imagegen_progress_bar'),
              value: controller.imageGenProgress,
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Step ${controller.imageGenCurrentStep} / ${controller.imageGenTotalSteps}',
                  key: const Key('imagegen_step_counter'),
                ),
                TextButton.icon(
                  key: const Key('cancel_imagegen_button'),
                  icon: const Icon(Icons.cancel, color: Colors.red),
                  label: const Text('Cancel', style: TextStyle(color: Colors.red)),
                  onPressed: () => controller.cancelImageGeneration(),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildGenerateActionBar() {
    final controller = widget.controller;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (controller.imageGenErrorMessage != null) ...[
          Container(
            key: const Key('imagegen_error_banner'),
            padding: const EdgeInsets.all(12),
            margin: const EdgeInsets.only(bottom: 12),
            decoration: BoxDecoration(
              color: Colors.red.withAlpha(25),
              border: Border.all(color: Colors.red.withAlpha(100)),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text(
              controller.imageGenErrorMessage!,
              style: const TextStyle(color: Colors.red),
            ),
          ),
        ],
        ElevatedButton.icon(
          key: const Key('generate_image_button'),
          style: ElevatedButton.styleFrom(
            padding: const EdgeInsets.symmetric(vertical: 16),
            backgroundColor: Theme.of(context).colorScheme.primary,
            foregroundColor: Theme.of(context).colorScheme.onPrimary,
          ),
          icon: const Icon(Icons.auto_awesome),
          label: Text(
            controller.isGeneratingImage ? 'Generating...' : 'Synthesize Image',
            style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
          ),
          onPressed: controller.isGeneratingImage
              ? null
              : () {
                  controller.generateImage(simulateTicks: true);
                },
        ),
      ],
    );
  }

  Widget _buildGallerySection() {
    final controller = widget.controller;
    final images = controller.galleryImages;

    return Column(
      key: const Key('canvas_gallery_view'),
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            const Icon(Icons.collections_outlined, color: Colors.purple),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                'Canvas Gallery (${images.length})',
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
              ),
            ),
          ],
        ),
        const SizedBox(height: 12),
        if (images.isEmpty)
          const Padding(
            padding: EdgeInsets.all(24.0),
            child: Center(child: Text('No generated images in gallery yet.')),
          )
        else
          ListView.separated(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            itemCount: images.length,
            separatorBuilder: (_, __) => const SizedBox(height: 12),
            itemBuilder: (context, idx) {
              final img = images[idx];
              final isSelected = controller.selectedGalleryImage?.id == img.id;

              return Card(
                key: Key('gallery_image_item_${img.id}'),
                elevation: isSelected ? 3 : 1,
                shape: RoundedRectangleBorder(
                  side: BorderSide(
                    color: isSelected
                        ? Theme.of(context).colorScheme.primary
                        : Colors.transparent,
                    width: 2,
                  ),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Padding(
                  padding: const EdgeInsets.all(12.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Container(
                            width: 48,
                            height: 48,
                            decoration: BoxDecoration(
                              color: Colors.deepPurple.withAlpha(40),
                              borderRadius: BorderRadius.circular(8),
                            ),
                            child: const Icon(Icons.image, color: Colors.deepPurple),
                          ),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  img.prompt,
                                  style: const TextStyle(fontWeight: FontWeight.bold),
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                ),
                                const SizedBox(height: 4),
                                Wrap(
                                  spacing: 6,
                                  children: [
                                    Chip(
                                      key: Key('gallery_mode_chip_${img.id}'),
                                      label: Text(img.mode.displayName),
                                      visualDensity: VisualDensity.compact,
                                    ),
                                    Chip(
                                      label: Text('${img.width}x${img.height}'),
                                      visualDensity: VisualDensity.compact,
                                    ),
                                    Chip(
                                      label: Text('Seed: ${img.seed}'),
                                      visualDensity: VisualDensity.compact,
                                    ),
                                  ],
                                ),
                              ],
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 12),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.end,
                        children: [
                          OutlinedButton.icon(
                            key: Key('view_provenance_button_${img.id}'),
                            icon: const Icon(Icons.fingerprint, size: 16),
                            label: const Text('Provenance'),
                            onPressed: () => _showProvenanceDialog(context, img),
                          ),
                          const SizedBox(width: 8),
                          IconButton(
                            key: Key('delete_gallery_image_button_${img.id}'),
                            icon: const Icon(Icons.delete_outline, color: Colors.red),
                            tooltip: 'Delete Image',
                            onPressed: () => controller.deleteGalleryImage(img.id),
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
    );
  }

  void _showProvenanceDialog(BuildContext context, GeneratedImageItem item) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        key: const Key('image_provenance_sheet'),
        title: const Row(
          children: [
            Icon(Icons.verified, color: Colors.green),
            SizedBox(width: 8),
            Text('Cryptographic Provenance'),
          ],
        ),
        content: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildProvenanceRow('Artifact ID:', item.id),
              _buildProvenanceRow('Mode:', item.mode.displayName),
              _buildProvenanceRow('Dimensions:', '${item.width} x ${item.height} px'),
              _buildProvenanceRow('Denoising Steps:', '${item.steps}'),
              _buildProvenanceRow('Guidance Scale:', '${item.guidanceScale}'),
              _buildProvenanceRow('Deterministic Seed:', '${item.seed}'),
              _buildProvenanceRow('File Size:', item.formattedSize),
              const Divider(height: 16),
              const Text('Prompt:', style: TextStyle(fontWeight: FontWeight.bold)),
              Text(item.prompt, style: const TextStyle(fontStyle: FontStyle.italic)),
              const SizedBox(height: 8),
              const Text('SHA-256 Digest:', style: TextStyle(fontWeight: FontWeight.bold)),
              SelectableText(
                item.sha256,
                key: const Key('provenance_sha256_text'),
                style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
              ),
              const SizedBox(height: 8),
              const Text('Summary Record:', style: TextStyle(fontWeight: FontWeight.bold)),
              Text(item.provenanceSummary, style: const TextStyle(fontSize: 12)),
            ],
          ),
        ),
        actions: [
          TextButton(
            key: const Key('close_provenance_dialog_button'),
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  Widget _buildProvenanceRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2.0),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: Colors.grey)),
          Text(value, style: const TextStyle(fontWeight: FontWeight.w600)),
        ],
      ),
    );
  }
}

class _MaskCanvasPainter extends CustomPainter {
  const _MaskCanvasPainter({
    required this.points,
    required this.brushSize,
  });

  final List<Offset> points;
  final double brushSize;

  @override
  void paint(Canvas canvas, Size size) {
    final bgPaint = Paint()..color = const Color(0xFF1E1E2E);
    canvas.drawRect(Rect.fromLTWH(0, 0, size.width, size.height), bgPaint);

    // Draw grid
    final gridPaint = Paint()
      ..color = Colors.white.withAlpha(20)
      ..strokeWidth = 1.0;
    for (double x = 0; x < size.width; x += 24) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), gridPaint);
    }
    for (double y = 0; y < size.height; y += 24) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), gridPaint);
    }

    // Draw mask points
    final maskPaint = Paint()
      ..color = Colors.white.withAlpha(200)
      ..strokeCap = StrokeCap.round
      ..strokeWidth = brushSize;

    for (final p in points) {
      canvas.drawCircle(p, brushSize / 2, maskPaint);
    }
  }

  @override
  bool shouldRepaint(covariant _MaskCanvasPainter oldDelegate) {
    return oldDelegate.points.length != points.length ||
        oldDelegate.brushSize != brushSize;
  }
}
