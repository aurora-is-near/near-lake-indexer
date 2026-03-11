import os
import re
from pathlib import Path

VERSION_PATTERN = r'^[\d\.]+\-[\d\.]+(\-[a-z]+\.\d+)?$'

def validate_version(version):
    """Validate that the version format is correct."""
    if not version:
        return False

    valid_version_pattern = re.compile(VERSION_PATTERN)
    return bool(valid_version_pattern.match(version))

def get_changelog_path():
    """Get changelog path."""
    custom_path = os.environ.get("CHANGELOG_PATH")
    changelog_path = Path(custom_path) if custom_path else Path("CHANGELOG.md")

    if not changelog_path.exists():
        print(f"Error: {changelog_path} not found")
        exit(1)

    return changelog_path

def extract_nearcore_version(version):
    """Extract nearcore version from the full version string."""
    return '-'.join(version.split('-')[1:]) if '-' in version else version

def update_changelog():
    new_version = os.environ.get("NEW_VERSION")

    if new_version and not validate_version(new_version):
        print(f"Error: Invalid version format '{new_version}'. Valid formats: 'x.y.z-a.b.c' or 'x.y.z-a.b.c-rc.n'")
        exit(1)

    if not new_version:
        print("Error: NEW_VERSION environment variable must be set")
        exit(1)

    changelog_path = get_changelog_path()
    content = changelog_path.read_text()

    # Check if version already exists
    version_pattern = re.compile(r'^# ' + re.escape(new_version) + r'$', re.MULTILINE)
    if version_pattern.search(content):
        print(f"Version {new_version} already exists in {changelog_path}, skipping update")
        return

    nearcore_version = extract_nearcore_version(new_version)
    new_entry = f"* chore: bump nearcore to {nearcore_version}"

    # Find the position after the "# Changelog" header
    lines = content.split('\n')

    # Find where to insert the new version
    insert_position = 1  # After "# Changelog"
    if len(lines) > 1 and lines[1] == '':
        insert_position = 2  # After "# Changelog" and empty line

    # Create new version section
    new_version_lines = [
        f"# {new_version}",
        new_entry,
        ""
    ]

    # Insert the new version section
    updated_lines = lines[:insert_position] + new_version_lines + lines[insert_position:]
    updated_content = '\n'.join(updated_lines)

    changelog_path.write_text(updated_content)
    print(f"Successfully updated {changelog_path} with new version {new_version}")

if __name__ == "__main__":
    update_changelog()
