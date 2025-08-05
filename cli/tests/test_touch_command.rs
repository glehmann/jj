// Copyright 2022 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::common::CommandOutput;
use crate::common::TestEnvironment;
use crate::common::TestWorkDir;

#[test]
fn test_touch() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    work_dir
        .run_jj(["bookmark", "create", "-r@", "a"])
        .success();
    work_dir.write_file("file1", "a\n");
    work_dir.run_jj(["new"]).success();
    work_dir
        .run_jj(["bookmark", "create", "-r@", "b"])
        .success();
    work_dir.write_file("file1", "b\n");
    work_dir.run_jj(["new"]).success();
    work_dir
        .run_jj(["bookmark", "create", "-r@", "c"])
        .success();
    work_dir.write_file("file1", "c\n");
    // Test the setup
    insta::assert_snapshot!(get_log(&work_dir), @r"
    @  mzvwutvlkqwt 2001-02-03 04:05:13.000 +07:00 c
    ○  kkmpptxzrspx 2001-02-03 04:05:11.000 +07:00 b
    ○  qpvuntsmwlqt 2001-02-03 04:05:09.000 +07:00 a
    ◆  zzzzzzzzzzzz 1970-01-01 00:00:00.000 +00:00
    [EOF]
    ");

    // Touch the commit (and it's descendant)
    let output = work_dir.run_jj(["touch", "kkmpptxzrspx"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Rebased 1 descendant commits
    Working copy  (@) now at: mzvwutvl b396a537 c | (no description set)
    Parent commit (@-)      : kkmpptxz 53f5eea6 b | (no description set)
    [EOF]
    ");
    insta::assert_snapshot!(get_log(&work_dir), @r"
    @  mzvwutvlkqwt 2001-02-03 04:05:14.000 +07:00 c
    ○  kkmpptxzrspx 2001-02-03 04:05:14.000 +07:00 b
    ○  qpvuntsmwlqt 2001-02-03 04:05:09.000 +07:00 a
    ◆  zzzzzzzzzzzz 1970-01-01 00:00:00.000 +00:00
    [EOF]
    ");

    // Reset change-id
    let output = work_dir.run_jj(["touch", "--reset-id", "kkmpptxzrspx"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Rebased 1 descendant commits
    Working copy  (@) now at: mzvwutvl 14fdd03c c | (no description set)
    Parent commit (@-)      : znkkpsqq de85401a b | (no description set)
    [EOF]
    ");
    insta::assert_snapshot!(get_log(&work_dir), @r"
    @  mzvwutvlkqwt 2001-02-03 04:05:16.000 +07:00 c
    ○  znkkpsqqskkl 2001-02-03 04:05:16.000 +07:00 b
    ○  qpvuntsmwlqt 2001-02-03 04:05:09.000 +07:00 a
    ◆  zzzzzzzzzzzz 1970-01-01 00:00:00.000 +00:00
    [EOF]
    ");
}

#[test]
fn test_squash_option_mutual_exclusion() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    work_dir.run_jj(["commit", "-m=a"]).success();
    work_dir.run_jj(["describe", "-m=b"]).success();
    insta::assert_snapshot!(work_dir.run_jj([
        "touch",
        "--author=Alice <alice@example.com>",
        "--reset-author",
    ]), @r"
    ------- stderr -------
    error: the argument '--author <AUTHOR>' cannot be used with '--reset-author'

    Usage: jj touch --author <AUTHOR> [REVSETS]...

    For more information, try '--help'.
    [EOF]
    [exit status: 2]
    ");
}

#[must_use]
fn get_log(work_dir: &TestWorkDir) -> CommandOutput {
    let template =
        r#"separate(" ", change_id.short(), committer.timestamp(), local_bookmarks, description)"#;
    work_dir.run_jj(["log", "-T", template])
}
