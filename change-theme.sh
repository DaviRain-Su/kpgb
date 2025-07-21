#!/bin/bash
# Theme switcher for KPGB

echo "🎨 KPGB Theme Switcher"
echo "Available themes:"
echo "  1) default - Classic blog design"
echo "  2) hacker - Terminal/Matrix style (green on black)"
echo "  3) minimal - Clean and modern"
echo "  4) dark - Modern dark mode"
echo "  5) cyberpunk - Neon and glitch effects"
echo ""
read -p "Select theme (1-5): " choice

case $choice in
    1) theme="default" ;;
    2) theme="hacker" ;;
    3) theme="minimal" ;;
    4) theme="dark" ;;
    5) theme="cyberpunk" ;;
    *) echo "Invalid choice!"; exit 1 ;;
esac

# Update site.toml
sed -i '' "s/^theme = .*/theme = \"$theme\"  # Options: default, hacker, minimal, dark, cyberpunk/" site.toml

echo "✅ Theme changed to: $theme"
echo ""
echo "To see the changes:"
echo "  - For static site: cargo run -- generate"
echo "  - For dev server: ./serve-dev.sh"
echo "  - For preview: ./preview.sh"