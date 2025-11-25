## [4.1.3](https://github.com/chess-seventh/rusty-commit-saver/compare/v4.1.2...v4.1.3) (2025-11-25)



## [4.1.2](https://github.com/chess-seventh/rusty-commit-saver/compare/v4.1.1...v4.1.2) (2025-11-25)



## [4.1.1](https://github.com/chess-seventh/rusty-commit-saver/compare/v4.1.0...v4.1.1) (2025-11-25)



# [4.1.0](https://github.com/chess-seventh/rusty-commit-saver/compare/v4.0.1...v4.1.0) (2025-11-25)


### Features

* Enhance dev environment with auto tools ([4bb79e5](https://github.com/chess-seventh/rusty-commit-saver/commit/4bb79e570303c7b06385234cefe3e904ec3fbdee))



## [4.0.1](https://github.com/chess-seventh/rusty-commit-saver/compare/v4.0.0...v4.0.1) (2025-11-25)



# [4.0.0](https://github.com/chess-seventh/rusty-commit-saver/compare/v3.2.0...v4.0.0) (2025-11-25)


* feat!: Update dev environment and dependencies ([c809d9f](https://github.com/chess-seventh/rusty-commit-saver/commit/c809d9f45fc65a21247c5f29dcdf747d96a667a5))


### BREAKING CHANGES

* update the rust channel to nightly



# [3.2.0](https://github.com/chess-seventh/rusty-commit-saver/compare/v3.1.1...v3.2.0) (2025-11-21)


### Features

* Update rust-overlay dependency in flake.lock ([c5c1d6c](https://github.com/chess-seventh/rusty-commit-saver/commit/c5c1d6c04b37688f0e40760e1f14298f457ab657))



## [3.1.1](https://github.com/chess-seventh/rusty-commit-saver/compare/v3.1.0...v3.1.1) (2025-10-03)



# [3.1.0](https://github.com/chess-seventh/rusty-commit-saver/compare/v3.0.0...v3.1.0) (2025-09-16)


### Features

* Enhance diary path handling and logging ([b2565d3](https://github.com/chess-seventh/rusty-commit-saver/commit/b2565d382d85613c852373e8e84a4e2948db5d32))



# [3.0.0](https://github.com/chess-seventh/rusty-commit-saver/compare/v2.2.1...v3.0.0) (2025-09-16)


* Clap Improvement and Logging (#9) ([7d00fad](https://github.com/chess-seventh/rusty-commit-saver/commit/7d00fad9a0be2a76babdfef92a11506a8ba2bba8)), closes [#9](https://github.com/chess-seventh/rusty-commit-saver/issues/9)


### BREAKING CHANGES

* Adds Clap support to define configuration values

* refactor: Streamline date handling in config

- Remove `chrono` dependency and related `today` field from `GlobalVars`
- Update `set_all` function and test cases accordingly
- Clean up unused code and comments from `main.rs`

* feat: Enhance logging and error handling across app

- Remove `emojis` dependency and integrate `env_logger` to enhance
  logging capabilities across various modules.
- Standardize error and debug logging in key components such as
  `vim_commit.rs` and `src/config.rs`.
- Enhance configuration management and dynamic path resolution using
  structured path construction from `GlobalVars`.
- Refactor integration tests by disabling certain tests and commenting
  out code sections related to diary file manipulations.
- Improve error handling and logging details during configuration
  retrieval and path checks.



## [2.2.1](https://github.com/chess-seventh/rusty-commit-saver/compare/v2.2.0...v2.2.1) (2025-09-12)



# [2.2.0](https://github.com/chess-seventh/rusty-commit-saver/compare/v2.1.0...v2.2.0) (2025-09-12)


### Features

* Update Rusty Commit Saver config and deps ([decdf6a](https://github.com/chess-seventh/rusty-commit-saver/commit/decdf6a826e63c88dd5d3ca68cdc8fd123db7ee1))



# [2.1.0](https://github.com/chess-seventh/rusty-commit-saver/compare/v2.0.2...v2.1.0) (2025-09-12)


### Features

* Streamline commit message extraction ([83d0783](https://github.com/chess-seventh/rusty-commit-saver/commit/83d0783263d802149e97b1b0665324c0427bb2a8))



## [2.0.2](https://github.com/chess-seventh/rusty-commit-saver/compare/v2.0.1...v2.0.2) (2025-09-12)



## [2.0.1](https://github.com/chess-seventh/rusty-commit-saver/compare/v2.0.0...v2.0.1) (2025-09-12)



# [2.0.0](https://github.com/chess-seventh/rusty-commit-saver/compare/v1.0.0...v2.0.0) (2025-09-12)


* feat!: Integrate Rusty Commit Saver in devenv ([e28def7](https://github.com/chess-seventh/rusty-commit-saver/commit/e28def7bad46af8db27b70e4912eb72717718517))


### BREAKING CHANGES

* Add my own flake (this one) to my devenv git-hooks



# [1.0.0](https://github.com/chess-seventh/rusty-commit-saver/compare/v0.2.4...v1.0.0) (2025-09-12)


* feat!: Rename package to 'rusty-commit-saver' ([cde93cd](https://github.com/chess-seventh/rusty-commit-saver/commit/cde93cd0c6ac6d00fc5345d951a07c88fa33f1f2))


### BREAKING CHANGES

* Rename the crate to rusty-commit-saver.



## [0.2.4](https://github.com/chess-seventh/rusty-commit-saver/compare/v0.2.3...v0.2.4) (2025-09-12)



## [0.2.3](https://github.com/chess-seventh/rusty-commit-saver/compare/v0.2.2...v0.2.3) (2025-09-12)



## [0.2.2](https://github.com/chess-seventh/rusty-commit-saver/compare/v0.2.1...v0.2.2) (2025-09-12)



## [0.2.1](https://github.com/chess-seventh/rusty-commit-saver/compare/v0.2.0...v0.2.1) (2025-09-12)



# [0.2.0](https://github.com/chess-seventh/rusty-commit-saver/compare/f44957ce49a7978a846b04b170bddef9c96f75ed...v0.2.0) (2025-09-12)


### Bug Fixes

* adapt vimwiki work default day entry ([6a7d1ab](https://github.com/chess-seventh/rusty-commit-saver/commit/6a7d1ab63e7b0e91638a5b1ef83f9229e641df10))
* clippy improvements ([31f4554](https://github.com/chess-seventh/rusty-commit-saver/commit/31f4554c968d7d236d794481054e41be73638031))
* double vimwiki ([b4b2cdb](https://github.com/chess-seventh/rusty-commit-saver/commit/b4b2cdb4d9a8f772c636b205bd292518b72db992))
* emoji ([426bc28](https://github.com/chess-seventh/rusty-commit-saver/commit/426bc28814c3c6d8e6d091ff7ee6ff28895f3f17))
* emoji ([97451a6](https://github.com/chess-seventh/rusty-commit-saver/commit/97451a69ad57cbe8902b940eb36004f85cf0aa6a))
* journal diary path ([03a3234](https://github.com/chess-seventh/rusty-commit-saver/commit/03a3234883d9e209754931bc238d02b4860d85f2))
* markup renderer ([52bfe2a](https://github.com/chess-seventh/rusty-commit-saver/commit/52bfe2ad05bb2b2154b5379a7253939b0c68f10a))
* markup renderer frontmatter tags ([ce59742](https://github.com/chess-seventh/rusty-commit-saver/commit/ce597423a309d54f09d12206a0e4c06f4ec9609b))
* path ([6bc9670](https://github.com/chess-seventh/rusty-commit-saver/commit/6bc9670f0883ed746ec232b8ac77a6089569d0ca))
* template renderer ([f7c827e](https://github.com/chess-seventh/rusty-commit-saver/commit/f7c827ee6b6df5047ce6c28189f4c0d4758957b7))
* template renderer for frontmatter ([96f6a39](https://github.com/chess-seventh/rusty-commit-saver/commit/96f6a3929475a4a84bd6cec5ad4ff5fef455f0d3))
* template renderer for frontmatter ([5a97266](https://github.com/chess-seventh/rusty-commit-saver/commit/5a972665dab0451d90cd3bbe14a7d1b0cf4582e3))
* template with path ([f3c8c0e](https://github.com/chess-seventh/rusty-commit-saver/commit/f3c8c0e03c87c28ecd0de54920309cac77a5ef00))
* typo ([2671355](https://github.com/chess-seventh/rusty-commit-saver/commit/26713553523eb584f182340c403ccba139bbf2c4))
* wip for debug struct ([a243c07](https://github.com/chess-seventh/rusty-commit-saver/commit/a243c07286054580be2e5e15db96903eb9909921))


### Features

* add cargo and git2 crate ([f44957c](https://github.com/chess-seventh/rusty-commit-saver/commit/f44957ce49a7978a846b04b170bddef9c96f75ed))
* add devenv.nix and flake.nix ([0005348](https://github.com/chess-seventh/rusty-commit-saver/commit/0005348563f48c62760032b179be6554ada2de92))
* add emojis crate to render ([b135873](https://github.com/chess-seventh/rusty-commit-saver/commit/b135873a71be3d3d683aff35a8e915085e3b641b))
* add loggin crate ([7809cc4](https://github.com/chess-seventh/rusty-commit-saver/commit/7809cc416584bbd5a0c0583e1424d75b9e85db0b))
* add shell.nix and envrc ([153f392](https://github.com/chess-seventh/rusty-commit-saver/commit/153f392f4d954cdd01fee58e50204956e8e8ecfd))
* change vault diary path containing emojis ([e954703](https://github.com/chess-seventh/rusty-commit-saver/commit/e954703b04a1932b136b09b7c4299719b2361ad1))
* change vimwiki path for work & home ([d1c0b2b](https://github.com/chess-seventh/rusty-commit-saver/commit/d1c0b2b58d97162749ade8275ba0c6e0a339c995))
* clippy improvements and fix templating issue for work ([904bce7](https://github.com/chess-seventh/rusty-commit-saver/commit/904bce72cd37017c7bff999ef78a55700a4ee0ec))
* continue working on base methods ([2eff454](https://github.com/chess-seventh/rusty-commit-saver/commit/2eff4543cbce511573e870fd671c8417e63372c9))
* do not use hook when in vimwiki dir and add std:env ([e7a5eaa](https://github.com/chess-seventh/rusty-commit-saver/commit/e7a5eaa2db2aca87a9212f16a13770eb76ef85e1))
* final draft and ready to test ([f8aeb86](https://github.com/chess-seventh/rusty-commit-saver/commit/f8aeb86f4c1322fcd67ea376fe7c524eb0b659bc))
* first draft ([edef697](https://github.com/chess-seventh/rusty-commit-saver/commit/edef6973077e7fa89c34d042b539a1c4248fd19b))
* fix all issues ([913e725](https://github.com/chess-seventh/rusty-commit-saver/commit/913e725a7dcf68bddc62bff801391d6fe2d1110e))
* fix treefmt git-hook ([b864c2e](https://github.com/chess-seventh/rusty-commit-saver/commit/b864c2e268968358fb9f87b20ddb164375ac9a74))
* major refactor ([9f0aa5a](https://github.com/chess-seventh/rusty-commit-saver/commit/9f0aa5af94bd6fd60ab3901290d8319114f035d0))
* move methods from VimRC struct as they don't need internal variables ([de4e214](https://github.com/chess-seventh/rusty-commit-saver/commit/de4e214e19956ecbbed0375dda7707c56ef2605e))
* split into multiple files to make the code more readable ([b88fdba](https://github.com/chess-seventh/rusty-commit-saver/commit/b88fdba9139384c43dcef2771339c8a5fb8e96c2))
* trim all commits to 120 chars ([84706bc](https://github.com/chess-seventh/rusty-commit-saver/commit/84706bc3e33e1686c5a0a14810bb3d11d320fdb4))
* Update YAML formatting in treefmt.toml ([e7241c7](https://github.com/chess-seventh/rusty-commit-saver/commit/e7241c78e9dcce6a8b7c31297cadeefe95a461f9))



