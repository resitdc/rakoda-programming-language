import 'package:flutter/material.dart';

enum ActivityType {
  explorer,
  search,
  browser,
  http,
  database,
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
            icon: Icons.file_copy_outlined,
            tooltip: 'Explorer',
          ),
          _buildIcon(
            context,
            type: ActivityType.search,
            icon: Icons.search,
            tooltip: 'Search',
          ),
          _buildIcon(
            context,
            type: ActivityType.browser,
            icon: Icons.public,
            tooltip: 'Browser',
          ),
          _buildIcon(
            context,
            type: ActivityType.http,
            icon: Icons.api,
            tooltip: 'HTTP Client',
          ),
          _buildIcon(
            context,
            type: ActivityType.database,
            icon: Icons.storage,
            tooltip: 'Database',
          ),
          const Spacer(),
          _buildIcon(
            context,
            type: null,
            icon: Icons.settings_outlined,
            tooltip: 'Settings',
            onTap: () {},
          ),
          const SizedBox(height: 8),
        ],
      ),
    );
  }

  Widget _buildIcon(
    BuildContext context, {
    required ActivityType? type,
    required IconData icon,
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
          child: Icon(
            icon,
            size: 22,
            color: isActive ? Colors.white : const Color(0xFF858585),
          ),
        ),
      ),
    );
  }
}
