# Instructions for setting up GH_TOKEN for release-please

## Create Personal Access Token (PAT)

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Set these permissions:
   - `repo` (Full control of private repositories)
   - `workflow` (Update GitHub Action workflows)
4. Generate the token and copy it immediately

## Add as Repository Secret

1. Go to your repository → Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `GH_TOKEN`
4. Value: Paste the PAT you created
5. Click "Add secret"

## Why This is Needed

GitHub Actions doesn't have permission to create PRs by default. Using a PAT with repository permissions allows release-please to:
- Create release branches
- Draft pull requests for releases
- Update CHANGELOG.md
- Manage version tags

After adding the secret, release-please will be able to create PRs for version bumps and releases.