#include <assert.h>
#include <string.h>

#include "../src/forth_core.h"

static void expect_ok(ForthResult result) {
    assert(result.code == FORTH_OK);
}

static void test_recorded_source_replays_on_reload(void) {
    ForthMachine machine;
    ForthMachine restored;
    char output[128];

    forth_init(&machine);
    expect_ok(forth_record_line(&machine, ": sq dup * ;"));
    expect_ok(forth_record_line(&machine, "7 sq ."));

    forth_init(&restored);
    expect_ok(forth_load_source(&restored, forth_source(&machine)));
    expect_ok(forth_eval_line(&restored, "8 sq .", output, sizeof(output)));

    assert(strcmp(output, "64") == 0);
}

static void test_source_append_does_not_reexecute(void) {
    ForthMachine machine;
    char output[128];

    forth_init(&machine);
    expect_ok(forth_eval_line(&machine, "5", output, sizeof(output)));
    expect_ok(forth_append_source_line(&machine, "5"));
    expect_ok(forth_eval_line(&machine, ".", output, sizeof(output)));

    assert(strcmp(output, "5") == 0);
}

static void test_user_definition_executes(void) {
    ForthMachine machine;
    char output[128];

    forth_init(&machine);
    expect_ok(forth_load_source(&machine, ": sq dup * ;\n"));
    expect_ok(forth_eval_line(&machine, "7 sq .", output, sizeof(output)));

    assert(strcmp(output, "49") == 0);
}

static void test_if_else_then_executes(void) {
    ForthMachine machine;
    char output[128];

    forth_init(&machine);
    expect_ok(forth_load_source(&machine, ": abs dup 0 < if -1 * then ;\n"));
    expect_ok(forth_eval_line(&machine, "-5 abs .", output, sizeof(output)));

    assert(strcmp(output, "5") == 0);
}

static void test_else_branch_executes(void) {
    ForthMachine machine;
    char output[128];

    forth_init(&machine);
    expect_ok(forth_load_source(&machine, ": choose if 11 else 22 then ;\n"));
    expect_ok(forth_eval_line(&machine, "0 choose .", output, sizeof(output)));

    assert(strcmp(output, "22") == 0);
}

static void test_begin_while_repeat_executes(void) {
    ForthMachine machine;
    char output[128];

    forth_init(&machine);
    expect_ok(forth_load_source(
        &machine,
        ": countdown begin dup while dup . 1 - repeat drop ;\n"));
    expect_ok(forth_eval_line(&machine, "3 countdown", output, sizeof(output)));

    assert(strcmp(output, "3 2 1") == 0);
}

static void test_begin_until_executes(void) {
    ForthMachine machine;
    char output[128];

    forth_init(&machine);
    expect_ok(forth_load_source(
        &machine,
        ": loopdown begin dup . 1 - dup 0 < until drop ;\n"));
    expect_ok(forth_eval_line(&machine, "2 loopdown", output, sizeof(output)));

    assert(strcmp(output, "2 1 0") == 0);
}

int main(void) {
    test_recorded_source_replays_on_reload();
    test_source_append_does_not_reexecute();
    test_user_definition_executes();
    test_if_else_then_executes();
    test_else_branch_executes();
    test_begin_while_repeat_executes();
    test_begin_until_executes();
    return 0;
}
