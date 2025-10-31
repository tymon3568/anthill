#!/bin/bash

echo "🐜 Anthill - GitHub Setup Script"
echo "================================"
echo ""

# Check if remote already exists
if git remote -v | grep -q "origin"; then
    echo "✅ Git remote 'origin' already exists:"
    git remote -v
    echo ""
    read -p "Do you want to remove and re-add it? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git remote remove origin
        echo "🗑️  Removed existing remote"
    else
        echo "ℹ️  Keeping existing remote. You can push with: git push -u origin master"
        exit 0
    fi
fi

echo "Please create a GitHub repository first:"
echo "1. Go to https://github.com/new"
echo "2. Repository name: anthill"
echo "3. Description: 🐜 Multi-tenant Inventory SaaS Platform - Rust + SvelteKit"
echo "4. Choose Public or Private"
echo "5. DO NOT initialize with README/license (we already have them)"
echo ""

read -p "Have you created the repository? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Please create the repository first, then run this script again."
    exit 1
fi

echo ""
echo "Choose connection method:"
echo "1. SSH (recommended) - git@github.com:username/anthill.git"
echo "2. HTTPS - https://github.com/username/anthill.git"
echo ""
read -p "Enter choice (1 or 2): " choice

echo ""
if [ "$choice" = "1" ]; then
    read -p "Enter your GitHub username: " username
    REPO_URL="git@github.com:$username/anthill.git"
elif [ "$choice" = "2" ]; then
    read -p "Enter your GitHub username: " username
    REPO_URL="https://github.com/$username/anthill.git"
else
    echo "❌ Invalid choice"
    exit 1
fi

echo ""
echo "📝 Adding remote: $REPO_URL"
git remote add origin "$REPO_URL"

if [ $? -eq 0 ]; then
    echo "✅ Remote added successfully!"
    echo ""
    echo "Current remotes:"
    git remote -v
    echo ""
    echo "🚀 Ready to push! Run:"
    echo "   git push -u origin master"
    echo ""
    read -p "Push now? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        echo "🚀 Pushing to GitHub..."
        git push -u origin master

        if [ $? -eq 0 ]; then
            echo ""
            echo "🎉 Success! Your repository is now on GitHub!"
            echo "   View at: https://github.com/$username/anthill"
        else
            echo ""
            echo "❌ Push failed. Common issues:"
            echo "   - SSH key not configured (for SSH method)"
            echo "   - Authentication failed (for HTTPS method)"
            echo "   - Repository doesn't exist"
            echo ""
            echo "Try pushing manually with: git push -u origin master"
        fi
    fi
else
    echo "❌ Failed to add remote"
    exit 1
fi
