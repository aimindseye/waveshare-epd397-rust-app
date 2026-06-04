# Create and upload the GitHub repository

The commands below create a new Git repository from this cleaned source tree and publish it with GitHub CLI.

## 1. Extract the clean ZIP

```bash
unzip waveshare-epd397-rust-app-v0.15.0-github-ready.zip
cd waveshare-epd397-rust-app
```

## 2. Validate the source tree

```bash
./scripts/validate.sh
./scripts/test-host.sh
source "$HOME/export-esp.sh"
./scripts/build.sh
```

## 3. Initialize Git and make the first commit

```bash
git init
git branch -M main
git add .
git commit -m "Initial RustMix Wave v0.15.0 release"
```

## 4. Install and authenticate GitHub CLI

```bash
brew install gh
gh auth login
```

## 5. Create the GitHub repository and push

For a public repository under the `aimindseye` account:

```bash
gh repo create aimindseye/waveshare-epd397-rust-app \
  --public \
  --source=. \
  --remote=origin \
  --push
```

Use `--private` instead of `--public` when the repository should not be public.

## 6. Optional release tag

```bash
git tag -a v0.15.0 -m "RustMix Wave v0.15.0"
git push origin v0.15.0
```
