# mgit

[![Crates.io](https://img.shields.io/crates/v/mgit.svg)](https://crates.io/crates/mgit)
[![Crates.io](https://img.shields.io/crates/l/mgit.svg)](https://github.com/koozz/mgit/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/mgit.svg)](https://crates.io/crates/mgit)

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

## Output

Per (found) git repository, the output will show:

* The path of the repository on disk.
* The output for the git action on that repository.

As a summary it will show some statistics:

```sh
Success: 110, Warnings: 3
```

## Roadmap / ideas

* Use async.
* Benchmark (and optimize?).

## Changelog

### 0.1.7

* Updated libc.

### 0.1.6

* Updated walkdir dependency.

### 0.1.5

* Add empty line at the end of the output.

### 0.1.4

* Updated dependencies.

### 0.1.3

* Output the amount of projects affected (and the command) up front.

### 0.1.2

* Moved output collection into spawned thread to release file handles earlier.

### 0.1.1

* Add a summary at the end of the output.

### 0.1.0

* Initial working version.

## License

[MIT licensed](./LICENSE)
