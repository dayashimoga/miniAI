import 'package:flutter/material.dart';
import '../controllers/platform_controller.dart';
import 'chat_workspace_screen.dart';
import 'dashboard_screen.dart';
import 'event_inspector_view.dart';
import 'hardware_inspector_screen.dart';
import 'model_manager_screen.dart';
import 'settings_screen.dart';
import 'training_studio_screen.dart';
import 'knowledge_base_screen.dart';
import 'sound_studio_screen.dart';
import 'video_studio_screen.dart';
import 'vision_canvas_screen.dart';
import 'consent_inspector_screen.dart';

class ShellScreen extends StatefulWidget {
  const ShellScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  State<ShellScreen> createState() => _ShellScreenState();
}

class _ShellScreenState extends State<ShellScreen> with WidgetsBindingObserver {
  PlatformController get controller => widget.controller;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didHaveMemoryPressure() {
    super.didHaveMemoryPressure();
    widget.controller.unloadActiveModel();
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.controller,
      builder: (context, _) {
        final activeScreen = _buildActiveScreen(widget.controller.selectedNavigationIndex);
        final screenWidth = MediaQuery.sizeOf(context).width;
        final isCompact = screenWidth < 600;

        return Scaffold(
          appBar: AppBar(
            title: Text(
              isCompact ? 'MiniAI' : 'MiniAI — Local-First Multimodal Platform',
            ),
            actions: [
              if (!isCompact)
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 4),
                  child: Semantics(
                    label: 'Active Workspace: ${widget.controller.activeWorkspaceId}',
                    child: ActionChip(
                      key: const Key('workspace_badge'),
                      avatar: const Icon(Icons.folder_outlined, size: 16),
                      label: Text(widget.controller.activeWorkspaceId),
                      onPressed: () {
                        widget.controller.setSelectedNavigationIndex(1);
                      },
                    ),
                  ),
                ),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 4),
                child: Semantics(
                  label: widget.controller.localOnlyMode
                      ? 'Privacy Status: Local-Only Privacy Mode Active'
                      : 'Privacy Status: Hybrid Consent Mode',
                  child: ActionChip(
                    key: const Key('privacy_status_badge'),
                    avatar: Icon(
                      controller.localOnlyMode
                          ? Icons.shield
                          : Icons.cloud_sync_outlined,
                      size: 18,
                    ),
                    label: isCompact
                        ? Text(controller.localOnlyMode ? 'LOCAL' : 'HYBRID')
                        : Text(
                            controller.localOnlyMode
                                ? 'LOCAL-ONLY PRIVACY'
                                : 'HYBRID CONSENT-GATED',
                          ),
                    onPressed: () {
                      controller.setSelectedNavigationIndex(11);
                    },
                  ),
                ),
              ),
              if (screenWidth >= 900)
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 4),
                  child: Semantics(
                    label:
                        'Hardware & Telemetry: CPU ${controller.telemetryCpuPercent}% RAM ${controller.peakMemoryRssMb.toStringAsFixed(0)}MB Latency ${controller.avgTtftMs.toStringAsFixed(0)}ms',
                    child: Container(
                      key: const Key('telemetry_status_bar'),
                      padding:
                          const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                      decoration: BoxDecoration(
                        color: Theme.of(context)
                            .colorScheme
                            .surfaceContainerHighest
                            .withValues(alpha: 0.5),
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(
                          color: Theme.of(context)
                              .colorScheme
                              .outline
                              .withValues(alpha: 0.2),
                        ),
                      ),
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          const Icon(Icons.speed,
                              size: 14, color: Color(0xFF14B8A6)),
                          const SizedBox(width: 4),
                          Text(
                            screenWidth >= 1280
                                ? 'FPS 60 | GPU 0% | CPU ${controller.telemetryCpuPercent}% | RAM ${controller.peakMemoryRssMb.toStringAsFixed(0)}MB | LAT ${controller.avgTtftMs.toStringAsFixed(0)}ms'
                                : 'CPU ${controller.telemetryCpuPercent}% | RAM ${controller.peakMemoryRssMb.toStringAsFixed(0)}MB',
                            style: const TextStyle(
                              fontSize: 11,
                              fontFamily: 'monospace',
                              fontWeight: FontWeight.w600,
                              letterSpacing: 0.3,
                            ),
                          ),
                          if (controller.isPowerThrottled) ...[
                            const SizedBox(width: 6),
                            const Icon(Icons.bolt, size: 13, color: Colors.amber),
                            const SizedBox(width: 2),
                            Text(
                              controller.batterySavingMode ? 'ECO' : 'THROTTLED',
                              style: const TextStyle(
                                fontSize: 10,
                                color: Colors.amber,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ],
                        ],
                      ),
                    ),
                  ),
                ),
              Builder(
                builder: (scaffoldCtx) => IconButton(
                  key: const Key('open_event_drawer_button'),
                  tooltip: 'Open Live Event Stream Drawer',
                  icon: Badge.count(
                    count: controller.events.length,
                    child: const Icon(Icons.stream),
                  ),
                  onPressed: () {
                    Scaffold.of(scaffoldCtx).openEndDrawer();
                  },
                ),
              ),
              const SizedBox(width: 8),
            ],
          ),
          endDrawer: EventInspectorView(
            controller: controller,
            isDrawer: true,
          ),
          body: LayoutBuilder(
            builder: (context, constraints) {
              final isWide = constraints.maxWidth >= 720;

              if (isWide) {
                return Row(
                  children: [
                    SingleChildScrollView(
                      child: ConstrainedBox(
                        constraints: BoxConstraints(minHeight: constraints.maxHeight),
                        child: IntrinsicHeight(
                          child: NavigationRail(
                            key: const Key('desktop_navigation_rail'),
                            selectedIndex: controller.selectedNavigationIndex,
                            onDestinationSelected: controller.setSelectedNavigationIndex,
                            labelType: NavigationRailLabelType.all,
                            destinations: const [
                        NavigationRailDestination(
                          icon: Icon(Icons.hub_outlined, key: Key('nav_rail_studio')),
                          selectedIcon: Icon(Icons.hub),
                          label: Text('Studio'),
                        ),
                        NavigationRailDestination(
                          icon: Icon(Icons.tune_outlined, key: Key('nav_rail_settings')),
                          selectedIcon: Icon(Icons.tune),
                          label: Text('Settings'),
                        ),
                        NavigationRailDestination(
                          icon: Icon(Icons.receipt_long_outlined, key: Key('nav_rail_events')),
                          selectedIcon: Icon(Icons.receipt_long),
                          label: Text('Events'),
                        ),
                        NavigationRailDestination(
                          icon: Icon(Icons.folder_copy_outlined, key: Key('nav_rail_models')),
                          selectedIcon: Icon(Icons.folder_copy),
                          label: Text('Models'),
                        ),
                        NavigationRailDestination(
                          icon: Icon(Icons.developer_board_outlined, key: Key('nav_rail_hardware')),
                          selectedIcon: Icon(Icons.developer_board),
                          label: Text('Hardware'),
                        ),
                        NavigationRailDestination(
                          icon: Icon(Icons.chat_bubble_outline, key: Key('nav_rail_chat')),
                          selectedIcon: Icon(Icons.chat_bubble),
                          label: Text('Chat'),
                        ),
                        NavigationRailDestination(
                          icon: Icon(Icons.model_training, key: Key('nav_rail_training')),
                          selectedIcon: Icon(Icons.model_training),
                          label: Text('Train'),
                        ),
                        NavigationRailDestination(
                          icon: Icon(Icons.auto_stories_outlined, key: Key('nav_rail_knowledge')),
                          selectedIcon: Icon(Icons.auto_stories),
                          label: Text('Knowledge'),
                        ),
                        NavigationRailDestination(
                          icon: Icon(Icons.palette_outlined, key: Key('nav_rail_canvas')),
                          selectedIcon: Icon(Icons.palette),
                          label: Text('Canvas'),
                        ),
                        NavigationRailDestination(
                          icon: Icon(Icons.graphic_eq_outlined, key: Key('nav_rail_sound')),
                          selectedIcon: Icon(Icons.graphic_eq),
                          label: Text('Sound'),
                        ),
                        NavigationRailDestination(
                          icon: Icon(Icons.movie_outlined, key: Key('nav_rail_video')),
                          selectedIcon: Icon(Icons.movie),
                          label: Text('Video'),
                        ),
                        NavigationRailDestination(
                          icon: Icon(Icons.cloud_sync_outlined, key: Key('nav_rail_cloud')),
                          selectedIcon: Icon(Icons.cloud_sync),
                          label: Text('Cloud'),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
              const VerticalDivider(thickness: 1, width: 1),
                    Expanded(child: activeScreen),
                  ],
                );
              }

              return activeScreen;
            },
          ),
          bottomNavigationBar: LayoutBuilder(
            builder: (context, constraints) {
              if (constraints.maxWidth >= 720) {
                return const SizedBox.shrink();
              }
              return NavigationBar(
                key: const Key('mobile_navigation_bar'),
                selectedIndex: controller.selectedNavigationIndex,
                onDestinationSelected: controller.setSelectedNavigationIndex,
                destinations: const [
                  NavigationDestination(
                    key: Key('nav_bar_studio'),
                    icon: Icon(Icons.hub_outlined),
                    selectedIcon: Icon(Icons.hub),
                    label: 'Studio',
                  ),
                  NavigationDestination(
                    key: Key('nav_bar_settings'),
                    icon: Icon(Icons.tune_outlined),
                    selectedIcon: Icon(Icons.tune),
                    label: 'Settings',
                  ),
                  NavigationDestination(
                    key: Key('nav_bar_events'),
                    icon: Icon(Icons.receipt_long_outlined),
                    selectedIcon: Icon(Icons.receipt_long),
                    label: 'Events',
                  ),
                  NavigationDestination(
                    key: Key('nav_bar_models'),
                    icon: Icon(Icons.folder_copy_outlined),
                    selectedIcon: Icon(Icons.folder_copy),
                    label: 'Models',
                  ),
                  NavigationDestination(
                    key: Key('nav_bar_hardware'),
                    icon: Icon(Icons.developer_board_outlined),
                    selectedIcon: Icon(Icons.developer_board),
                    label: 'Hardware',
                  ),
                  NavigationDestination(
                    key: Key('nav_bar_chat'),
                    icon: Icon(Icons.chat_bubble_outline),
                    selectedIcon: Icon(Icons.chat_bubble),
                    label: 'Chat',
                  ),
                  NavigationDestination(
                    key: Key('nav_bar_training'),
                    icon: Icon(Icons.model_training),
                    selectedIcon: Icon(Icons.model_training),
                    label: 'Train',
                  ),
                  NavigationDestination(
                    key: Key('nav_bar_knowledge'),
                    icon: Icon(Icons.auto_stories_outlined),
                    selectedIcon: Icon(Icons.auto_stories),
                    label: 'Knowledge',
                  ),
                  NavigationDestination(
                    key: Key('nav_bar_canvas'),
                    icon: Icon(Icons.palette_outlined),
                    selectedIcon: Icon(Icons.palette),
                    label: 'Canvas',
                  ),
                  NavigationDestination(
                    key: Key('nav_bar_sound'),
                    icon: Icon(Icons.graphic_eq_outlined),
                    selectedIcon: Icon(Icons.graphic_eq),
                    label: 'Sound',
                  ),
                  NavigationDestination(
                    key: Key('nav_bar_video'),
                    icon: Icon(Icons.movie_outlined),
                    selectedIcon: Icon(Icons.movie),
                    label: 'Video',
                  ),
                  NavigationDestination(
                    key: Key('nav_bar_cloud'),
                    icon: Icon(Icons.cloud_sync_outlined),
                    selectedIcon: Icon(Icons.cloud_sync),
                    label: 'Cloud',
                  ),
                ],
              );
            },
          ),
        );
      },
    );
  }

  Widget _buildActiveScreen(int index) {
    switch (index) {
      case 1:
        return SettingsScreen(controller: widget.controller);
      case 2:
        return EventInspectorView(controller: widget.controller);
      case 3:
        return ModelManagerScreen(controller: widget.controller);
      case 4:
        return HardwareInspectorScreen(controller: widget.controller);
      case 5:
        return ChatWorkspaceScreen(controller: widget.controller);
      case 6:
        return TrainingStudioScreen(controller: widget.controller);
      case 7:
        return KnowledgeBaseScreen(controller: widget.controller);
      case 8:
        return VisionCanvasScreen(controller: widget.controller);
      case 9:
        return SoundStudioScreen(controller: widget.controller);
      case 10:
        return VideoStudioScreen(controller: widget.controller);
      case 11:
        return ConsentInspectorScreen(controller: widget.controller);
      case 0:
      default:
        return DashboardScreen(controller: widget.controller, embedded: true);
    }
  }
}
