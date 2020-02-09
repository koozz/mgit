# mgit

Multi-git is a quick tool that could easily have been created with some shell
scripts, but it's a nice project to tackle a part of my daily workflow.
It performs git actions on multiple directories within the current tree.

It will:
* walk the directory tree;
* traverse and find all git projects;
* perform `git` with all the arguments you passed to  `mgit`.
* collect output per directory;
* outputting thread-safe in the main thread;

It can even be tweaked with the environment variable `MGIT_PARALLEL`
(defaulting to number of cores times 2).

## Examples

Keeping your indices up-to-date:

```sh
$ mgit fetch
```

Keeping your code up to date (if no conflicts):

```sh
$ mgit pull --ff-only
```

Keeping your code up to date, rebasing and using autostash:

```sh
$ mgit pull --rebase --autostash
```

## License

MIT
