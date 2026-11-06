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

class ShellScreen extends StatelessWidget {
  const ShellScreen({
    super.key,
    required this.controller,
  });

  final PlatformController controller;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        final activeScreen = _buildActiveScreen(controller.selectedNavigationIndex);
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
                    label: 'Active Workspace: ${controller.activeWorkspaceId}',
                    child: ActionChip(
                      key: const Key('workspace_badge'),
                      avatar: const Icon(Icons.folder_outlined, size: 16),
                      label: Text(controller.activeWorkspaceId),
                      onPressed: () {
                        controller.setSelectedNavigationIndex(1);
                      },
                    ),
                  ),
                ),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 4),
                child: Semantics(
                  label: controller.localOnlyMode
                      ? 'Privacy Status: Local-Only Privacy Mode Active'
                      : 'Privacy Status: Hybrid Consent Mode',
                  child: Chip(
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
                    NavigationRail(
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
                      ],
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
        return SettingsScreen(controller: controller);
      case 2:
        return EventInspectorView(controller: controller);
      case 3:
        return ModelManagerScreen(controller: controller);
      case 4:
        return HardwareInspectorScreen(controller: controller);
      case 5:
        return ChatWorkspaceScreen(controller: controller);
      case 6:
        return TrainingStudioScreen(controller: controller);
      case 7:
        return KnowledgeBaseScreen(controller: controller);
      case 0:
      default:
        return DashboardScreen(controller: controller, embedded: true);
    }
  }
}
