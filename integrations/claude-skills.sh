#!/usr/bin/env bash
set -euo pipefail

echo "=== Claude Skills Integration Installer ==="
echo ""

OPENCODE_DIR="${HOME}/.config/opencode"

if [ ! -d "$OPENCODE_DIR" ]; then
  echo "Error: opencode config directory not found at $OPENCODE_DIR"
  echo "Install opencode first: https://opencode.ai"
  exit 1
fi

SKILLS_DIR="${OPENCODE_DIR}/skills"

echo "Installing external skills from Jeffallan/claude-skills..."
echo "Repository: https://github.com/Jeffallan/claude-skills"
echo ""

# --- devops-engineer ---
DEVOPS_DIR="${SKILLS_DIR}/devops-engineer"
echo "1. devops-engineer (Docker, Kubernetes, Terraform, CI/CD, GitOps, incident response)..."
if [ -d "$DEVOPS_DIR" ]; then
  echo "   Already installed at $DEVOPS_DIR"
else
  echo "   Download instructions:"
  echo "   1. Clone: git clone https://github.com/Jeffallan/claude-skills /tmp/claude-skills"
  echo "   2. Copy: cp -r /tmp/claude-skills/skills/devops-engineer $DEVOPS_DIR"
  echo "   3. Clean: rm -rf /tmp/claude-skills"
  echo ""
  echo "   Or install via git subtree:"
  echo "   cd $OPENCODE_DIR"
  echo "   git clone --depth 1 --filter=blob:none --sparse https://github.com/Jeffallan/claude-skills /tmp/claude-skills-tmp"
  echo "   cp -r /tmp/claude-skills-tmp/skills/devops-engineer $DEVOPS_DIR"
  echo "   rm -rf /tmp/claude-skills-tmp"
  echo ""
  echo "   Used by: devops-agent during iteration zero and ready-to-deploy"
fi

# --- security-reviewer ---
SECURITY_DIR="${SKILLS_DIR}/security-reviewer"
echo ""
echo "2. security-reviewer (SAST, vulnerability scanning, penetration testing, secrets scanning)..."
if [ -d "$SECURITY_DIR" ]; then
  echo "   Already installed at $SECURITY_DIR"
else
  echo "   Download instructions:"
  echo "   1. Clone: git clone https://github.com/Jeffallan/claude-skills /tmp/claude-skills"
  echo "   2. Copy: cp -r /tmp/claude-skills/skills/security-reviewer $SECURITY_DIR"
  echo "   3. Clean: rm -rf /tmp/claude-skills"
  echo ""
  echo "   Or install via git sparse checkout (see devops-engineer above)"
  echo ""
  echo "   Used by: secops-agent for vulnerability audits and compliance checks"
fi

echo ""
echo "=== Claude Skills installation complete ==="
echo ""
echo "Next: These skills are referenced by devops-agent and secops-agent."
echo "  The agents will load them automatically when triggered."
echo ""
echo "Tip: To install all available skills from Jeffallan/claude-skills:"
echo "  git clone https://github.com/Jeffallan/claude-skills /tmp/claude-skills"
echo "  cp -r /tmp/claude-skills/skills/* $SKILLS_DIR/"
echo "  rm -rf /tmp/claude-skills"
