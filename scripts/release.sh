#/usr/bin/env bash

# All crates to publish; NOTE: Order matters here! Publish happens in exact
# order, so depenencies of each other must be published first! dev-dependencies
# are temporarily removed using cargo-hack
CRATES=(
  entity_noop_macros
  entity_macros_data
  entity_macros
  entity
  entity_inmemory
  entity_sled
  entity_async_graphql_macros
  entity_async_graphql
)

# https://stackoverflow.com/questions/59895/how-can-i-get-the-source-directory-of-a-bash-script-from-within-the-script-itsel
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
CHANGELOG="$DIR/../CHANGELOG.md"
ROOT_CARGO_TOML="$DIR/../Cargo.toml"
DRY_RUN=1
VERBOSE=0
QUIET=0
GIT_BRANCH=main
SKIP_CHANGELOG=0
SKIP_GIT_TAG=0
SKIP_CARGO_TOML_UPDATE=0
SKIP_CARGO_PUBLISH=0
SKIP_GIT_PUSH=0

# Supports OSX, Ubuntu, Freebsd, Cygwin, CentOS, Red Hat Enterprise, & Msys
# https://izziswift.com/sed-i-command-for-in-place-editing-to-work-with-both-gnu-sed-and-bsd-osx/
sedi () {
  if [ "$VERBOSE" -eq 1 ]; then
    sed "$@" > ".file.tmp.out"
    diff -u "${@: -1}" ".file.tmp.out"
    rm ".file.tmp.out"
  fi

  if [ "$DRY_RUN" -eq 0 ]; then
    sed --version >/dev/null 2>&1 && sed -i -- "$@" || sed -i "" "$@"

  # Special situation to use sed to return some value rather than modify a file
  elif [ "$1" = "{GET}" ]; then
    sed "${@:2}"
  fi
}

print_msg () {
  if [ "$QUIET" -eq 0 ]; then
    echo "$@"
  fi
}

# NOTE: We want to detect the current version without diff
TARGET_VERSION="$(sedi {GET} -n "s/^version = \"\([^\"]*\)\"/\1/p" "$ROOT_CARGO_TOML")"
NEXT_VERSION=

function usage {
  echo "Usage: $(basename $0) [-vfhq] [-s STEP] [-b BRANCH] [-t TARGET_VERSION] [-n NEXT_VERSION]" 2>&1
  echo 'Release the current version of entity crates.'
  echo ''
  echo "   -t VERSION  Specify target version to use (default = $TARGET_VERSION)"
  echo '   -n VERSION  Specify next version to update all Cargo.toml (does nothing if not provided)'
  echo '   -s STEP     Skips the specified step and can be supplied multiple times'
  echo '               Choices are changelog, git-tag, git-push, cargo-toml-update, cargo-publish'
  echo "   -b BRANCH   Specify git branch to push to (default = $GIT_BRANCH)"
  echo '   -f          Force release, rather than performing dry run'
  echo '   -h          Print this help information'
  echo '   -v          Increase verbosity'
  echo '   -q          Suppress output (quiet)'
  exit 1
}

while getopts ':vfhqt:n:s:b:' arg; do
  case "${arg}" in
    t) TARGET_VERSION=${OPTARG};;
    n) NEXT_VERSION=${OPTARG};;
    q) QUIET=1;;
    v) VERBOSE=1;;
    f) DRY_RUN=0;;
    b) GIT_BRANCH=${OPTARG};;
    s)
      case "${OPTARG}" in
        changelog) SKIP_CHANGELOG=1;;
        git-tag) SKIP_GIT_TAG=1;;
        git-push) SKIP_GIT_PUSH=1;;
        cargo-toml-update) SKIP_CARGO_TOML_UPDATE=1;;
        cargo-publish) SKIP_CARGO_PUBLISH=1;;
        *)
          echo "Unknown step to skip: ${OPTARG}"
          echo
          usage
          ;;
      esac
      ;;
    h)
      usage
      ;;
    :)
      echo "Option missing argument: -${OPTARG}"
      echo
      usage
      ;;
    ?)
      echo "Invalid option: -${OPTARG}"
      echo
      usage
      ;;
  esac
done

shift "$OPTIND"

# Update the changelog with our new version information
# 1. Replace unreleased with version being published
# 2. Replace release date with actual date in YYYY-MM-DD format
# 3. Add new unreleased template at top of changelog
# 4. Commit all changes in git
if [ "$SKIP_CHANGELOG" -eq 1 ]; then
  print_msg 'Skipping changelog updates!'
elif [ -n "$TARGET_VERSION" ]; then
  print_msg "[$TARGET_VERSION]: $CHANGELOG"
  sedi "s/Unreleased/$TARGET_VERSION/g" "$CHANGELOG"
  sedi "s/ReleaseDate/$(date "+%Y-%m-%d")/g" "$CHANGELOG"
  sedi "s/<!-- next-header -->/<!-- next-header -->\n\n## [Unreleased] - ReleaseDate/g" "$CHANGELOG"

  # If not dry-run, we will add the changelog updates as a new commit
  if [ "$DRY_RUN" -eq 0 ]; then
    git add --all
    git commit -m "[Release $TARGET_VERSION] Changelog updates"
  else
    print_msg 'git add --all'
    print_msg "git commit -m \"[Release $TARGET_VERSION] Changelog updates\""
  fi
else
  print_msg 'Target version not provided! Skipping CHANGELOG.md updates & tagging!'
fi

# Publish each crate with current version
if [ "$SKIP_CARGO_PUBLISH" -eq 0 ]; then
  for crate in "${CRATES[@]}"; do
    dry_run_arg=
    if [ "$DRY_RUN" -eq 1 ]; then
      dry_run_arg=--dry-run
    fi

    quiet_arg=
    if [ "$QUIET" -eq 1 ]; then
      quiet_arg=--quiet
    fi

    cargo hack publish -p "$crate" --no-dev-deps --allow-dirty $dry_run_arg $quiet_arg
  done
else
  print_msg 'Skipping Cargo crate publishing!'
fi

# Update all Cargo.toml with version change for entity-related crates
# 1. Replace crate's version with new version
# 2. Replace dependency crates' versions with new version
if [ "$SKIP_CARGO_TOML_UPDATE" -eq 1 ]; then
  print_msg 'Skipping Cargo.toml updates!'
elif [ -n "$NEXT_VERSION" ]; then
  CARGO_TOML_FILES=($(find "$DIR/.." -name "Cargo.toml"))
  for cargo_toml in "${CARGO_TOML_FILES[@]}"; do
    print_msg "[$TARGET_VERSION -> $NEXT_VERSION]: $cargo_toml"
    sedi "1,/^version/ s/^version = \".*\"/version = \"$NEXT_VERSION\"/g" "$cargo_toml"
    sedi "s/^\(entity.*version = \"\)[^\"]*\(\".*\)$/\1=$NEXT_VERSION\2/g" "$cargo_toml"
  done

  # If not dry-run, we will add the Cargo.toml updates as a new commit
  if [ "$DRY_RUN" -eq 0 ]; then
    git add --all
    git commit -m "[Release $TARGET_VERSION] Bump to next version ($NEXT_VERSION)"
  else
    print_msg 'git add --all'
    print_msg "git commit -m \"[Release $TARGET_VERSION] Bump to next version ($NEXT_VERSION)\""
  fi
else
  print_msg 'Next version not provided! Skipping Cargo.toml updates!'
fi

if [ "$SKIP_GIT_TAG" -eq 0 ]; then
  if [ "$DRY_RUN" -eq 0 ]; then
    git tag "v$TARGET_VERSION"
  else
    print_msg "git tag \"v$TARGET_VERSION\""
  fi
else
  print_msg 'Skipping git tagging!'
fi

if [ "$DRY_RUN" -eq 0 ]; then
  if [ "$SKIP_GIT_PUSH" -eq 0 ]; then
    git push origin "$GIT_BRANCH"
    if [ "$SKIP_GIT_TAG" -eq 0 ]; then
      git push origin "v$TARGET_VERSION"
    fi
  fi
else
  if [ "$SKIP_GIT_PUSH" -eq 0 ]; then
    print_msg "git push origin \"$GIT_BRANCH\""
    if [ "$SKIP_GIT_TAG" -eq 0 ]; then
      print_msg "git push origin \"v$TARGET_VERSION\""
    fi
  fi
fi
