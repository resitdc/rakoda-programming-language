import re

with open("apps/studio/lib/features/database/database_workspace.dart", "r") as f:
    content = f.read()

# Fix _fetchSuggestionsForQueryTab
old_fetch = """      if (mounted) {
        setState(() {
          _querySuggestions[tab.id] = suggestions.toList();
        });
      }
      await svc.disconnect();
    } catch (_) {}"""

new_fetch = """      if (mounted) {
        setState(() {
          _querySuggestions[tab.id] = suggestions.toList();
        });
      }
    } catch (_) {
    } finally {
      try {
        await DatabaseService.fromConnection(tab.connection).disconnect();
      } catch (_) {}
    }"""

if old_fetch in content:
    content = content.replace(old_fetch, new_fetch)

with open("apps/studio/lib/features/database/database_workspace.dart", "w") as f:
    f.write(content)
