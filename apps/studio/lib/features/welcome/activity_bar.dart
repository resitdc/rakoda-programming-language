import 'package:flutter/material.dart';
import 'package:hugeicons/hugeicons.dart';
import '../settings/settings_screen.dart';

enum ActivityType {
  explorer,
  search,
  browser,
  http,
  database,
  chat,
}

class ActivityBar extends StatelessWidget {
  final ActivityType? activeActivity;
  final ValueChanged<ActivityType> onActivitySelected;

  const ActivityBar({
    super.key,
    required this.activeActivity,
    required this.onActivitySelected,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 44,
      color: const Color(0xFF333333),
      child: Column(
        children: [
          const SizedBox(height: 4),
          _buildIcon(
            context,
            type: ActivityType.explorer,
            icon: HugeIcons.strokeRoundedFile02,
            tooltip: 'Explorer',
          ),
          _buildIcon(
            context,
            type: ActivityType.search,
            icon: HugeIcons.strokeRoundedSearch01,
            tooltip: 'Search',
          ),
          _buildIcon(
            context,
            type: ActivityType.browser,
            icon: HugeIcons.strokeRoundedGlobal,
            tooltip: 'Browser',
          ),
          _buildIcon(
            context,
            type: ActivityType.http,
            icon: HugeIcons.strokeRoundedApi,
            tooltip: 'HTTP Client',
          ),
          _buildIcon(
            context,
            type: ActivityType.database,
            icon: HugeIcons.strokeRoundedDatabase,
            tooltip: 'Database',
          ),
          _buildIcon(
            context,
            type: ActivityType.chat,
            icon: HugeIcons.strokeRoundedChatting01,
            tooltip: 'LAN Chat',
          ),
          const Spacer(),
          _buildIcon(
            context,
            type: null,
            icon: HugeIcons.strokeRoundedSettings02,
            tooltip: 'Settings',
            onTap: () {
              Navigator.of(context).push(
                MaterialPageRoute(builder: (context) => const SettingsScreen()),
              );
            },
          ),
          const SizedBox(height: 8),
        ],
      ),
    );
  }

  Widget _buildIcon(
    BuildContext context, {
    required ActivityType? type,
    required dynamic icon,
    required String tooltip,
    VoidCallback? onTap,
  }) {
    final isActive = activeActivity == type && type != null;
    return Tooltip(
      message: tooltip,
      preferBelow: false,
      child: InkWell(
        onTap: onTap ?? () {
          if (type != null) onActivitySelected(type);
        },
        child: Container(
          width: 44,
          height: 44,
          decoration: BoxDecoration(
            border: Border(
              left: BorderSide(
                color: isActive ? Colors.white : Colors.transparent,
                width: 2,
              ),
            ),
          ),
          child: Center(
            child: icon is IconData 
                ? Icon(
                    icon,
                    size: 20,
                    color: isActive ? Colors.white : const Color(0xFF858585),
                  )
                : HugeIcon(
                    icon: icon,
                    size: 20,
                    color: isActive ? Colors.white : const Color(0xFF858585),
                  ),
          ),
        ),
      ),
    );
  }
}
