[![marketplace](https://img.shields.io/badge/marketplace-version--generator-blue?logo=github)](https://github.com/marketplace/actions/version-generator)
[![CI](https://github.com/lpenz/ghaction-version-gen/actions/workflows/ci.yml/badge.svg)](https://github.com/lpenz/ghaction-version-gen/actions/workflows/ci.yml)
[![coveralls](https://coveralls.io/repos/github/lpenz/ghaction-version-gen/badge.svg?branch=main)](https://coveralls.io/github/lpenz/ghaction-version-gen?branch=main)
[![github](https://img.shields.io/github/v/release/lpenz/ghaction-version-gen?include_prereleases&label=release&logo=github)](https://github.com/lpenz/ghaction-version-gen/releases)
[![docker](https://img.shields.io/docker/v/lpenz/ghaction-version-gen?label=release&logo=docker&sort=semver)](https://hub.docker.com/repository/docker/lpenz/ghaction-version-gen)

# ghaction-version-gen

ghaction-version-gen is a docker github action that outputs a version
number for you to use in a deploy action.

There are many ways to generate version information for a
repository. They usually involve processing `GITHUB_REF` in some
[way](https://stackoverflow.com/questions/58177786/get-the-current-pushed-tag-in-github-actions),
maybe using even using [github-script](https://github.com/actions/github-script).

This repository is also an example of how to create a docker github
action that compiles a rust entrypoint in a container and then moves
it to a second, minimal container.


## Outputs

The following are the *primary* outputs of this action, usually the
ones used for versioning:

- `version_tagged`: for repositories that should only deploy on tags,
  it's defined if the github event was a push of a tag.

  The output itself is the tag, with the optional `v` stripped.

- `version_commit`: for repositories that deploy on tags and on all
  commits to `master` or `main`. It's defined if the github event was
  a push of a tag or of one of those branches.

  The output itself is the the tag that was pushed, or the most recent
  tag on the branch followed by the distance between the branch and
  the tag (always with the `v` stripped).

- `version_docker_ci`: for repositories that deploy to
  [hub.docker](http://hub.docker.com/) continuously. If the commit was
  pushed to `master` or `main`, the variable has the value *"latest"*;
  if a tag was pushed, it has the tag (`v` stripped); otherwise, it has
  the value *"null"* as a string. This last case allows us to always
  use the variable as the version for the [build-push-action], which
  doesn't like empty strings.


You can these variables in action in the [Examples](#examples) section.


### Secondary outputs

These are the *secondary* outputs that might be useful for debugging
or as alternative versioning schemes:

- `is_push`: if the github event was identified, "true" if the event
  was a push or "false" otherwise.
- `is_tag`: if the github ref was identified, "true" if the ref is a
  tag, false otherwise.
- `is_main`: "true" if the github ref was for a branch named `main` or
  `master`.
- `is_push_tag`: "true" if a tag was pushed.
- `is_push_main`: "true" if `main` or `master` were pushed.
- `commit`: the hash of the commit.
- `git_describe_tags`: the output of `git describe --tags`
- `tag_latest`: the most recent tag.
- `distance`: the distance between the current commit and `tag_latest`.
- `tag_distance`: `tag_latest-distance`
- `tag_head`: the tag on HEAD, if there's a tag on HEAD (does not
  depend on the gitub event).
- `dash_distance`: `-` prepended to `distance`
- `tag_latest_ltrimv`: `tag_latest` without the optional leading `v`.
- `tag_head_ltrimv`: `tag_head` without the optionsl leading `v`, if
  `tag_head` was defined.
- `version_tagged`: `tag_head_ltrimv` if `is_push_tag`.
- `version_commit`: `tag_head_ltrimv` if `is_push_tag` or
  `tag_distance_ltrimv` if `is_push_main`.
- `version_docker_ci`: *"latest"* if `is_push_main`, `tag_head_ltrimv`
  if `is_push_tag`.


## Examples

### `version_tagged` and `version_commit`

Using `version_tagged` and `version_commit` is quite simple:

```yml
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - id: version
        uses: docker://lpenz/ghaction-version-gen:0.6
      ...
      - name: deploy
        uses: <deploy action>
        if: steps.version.outputs.version_tagged != ''
        with:
          version: ${{ steps.version.outputs.version_tagged }}
```

That gets the *deploy* step to run only when a tag is pushed, and sets
`version` to the tag with the optional `v` prefix stripped.

If we replace `version_tagged` with `version_commit` in the example
above, we get an additional behavior: `version_commit` is also defined
when `main` or `master` are pushed, and it assumes the value of the
last tag (`v` stripped) suffixed with `-` and the distance of the
pushed commit to that tag. This should be used in projects that want
to deploy every time `main` is pushed.


### `version_docker_ci`

The `version_docker_ci` variable was designed to work with the
docker's [build-push-action].  It should be used in projects where we
would use want to deploy from tags and from `main`/`master`, but we
want `main`/`master` to be identified as `latest` in docker hub.

This is how it's used:

```yml
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - id: version
        uses: docker://lpenz/ghaction-version-gen:0.6
      - uses: docker/login-action@v1
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}
      - uses: docker/build-push-action@v2
        with:
          push: ${{ steps.version.outputs.version_docker_ci != 'null' }}
          tags: ${{ github.repository }}:${{ steps.version.outputs.version_docker_ci }}
```

Note that we don't make the [build-push-action] step conditional
because we always want to build the container. Instead, we make `push`
conditional, by checking that `version_docker_ci` is not `null`. We
use `null` instead of the empty string as a workaround, because the
action doesn't let us use an empty string as the version in `tags`.


[build-push-action]: https://github.com/marketplace/actions/build-and-push-docker-images

