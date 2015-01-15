#include <lua.h>
#include <lauxlib.h>
#include <lualib.h>  /* Prototype for luaL_openlibs(), */
#include <stdlib.h>
#include <stdio.h>

#include <time.h>
#include <sys/time.h>

#ifdef __MACH__
#include <mach/clock.h>
#include <mach/mach.h>
#endif

lua_State *L = 0;

enum DECISION {
    NOOP = 0,
    LONG = 1,
    SHORT = 2,
    CLOSE = 3
} trading_decisions;

int trading_decision;

void ensure_initialized() {
    if(!L) {
        fprintf(stderr, "\nLua must be initialized before calling this function\n");
        exit(1);
    }
}

void ensure_not_initialized() {
    if(L) {
        fprintf(stderr, "\nAttempted to initialize an existing Lua session\n");
        exit(1);
    }
}

int get_nanoseconds(lua_State *L) {
    struct timespec ts;

    #ifdef __MACH__ // OS X does not have clock_gettime, use clock_get_time
    clock_serv_t cclock;
    mach_timespec_t mts;
    host_get_clock_service(mach_host_self(), CALENDAR_CLOCK, &cclock);
    clock_get_time(cclock, &mts);
    mach_port_deallocate(mach_task_self(), cclock);
    ts.tv_sec = mts.tv_sec;
    ts.tv_nsec = mts.tv_nsec;
    #else
    clock_gettime(CLOCK_REALTIME, &ts);
    #endif

    return ts.tv_nsec;
}

int open_long_trade(lua_State *L) {
    trading_decision = LONG;
    return 0;
}

int open_short_trade(lua_State *L) {
    trading_decision = SHORT;
    return 0;
}

int close_trade(lua_State *L) {
    trading_decision = CLOSE;
    return 0;
}

int lua_get_decision() {
    return trading_decision;
}

void bail(lua_State *L, char *msg) {
    fprintf(stderr, "\nFATAL ERROR IN LUA:\n  %s: %s\n\n", msg, lua_tostring(L, -1));
    exit(1);
}

void lua_bridge_setup(char *path) {
    ensure_not_initialized();

    L = luaL_newstate();
    luaL_openlibs(L);

    // TODO: No performance benefit from this... yet.
    // lua_gc(L, LUA_GCSTOP, 0);

    // register C functions

    lua_pushcfunction(L, open_long_trade);
    lua_setglobal(L, "open_long_trade");

    lua_pushcfunction(L, open_short_trade);
    lua_setglobal(L, "open_short_trade");

    lua_pushcfunction(L, close_trade);
    lua_setglobal(L, "close_trade");

    lua_pushcfunction(L, get_nanoseconds);
    lua_setglobal(L, "get_nanoseconds");

    // seed PRNG with nanoseconds

    luaL_dostring(L, "math.randomseed(get_nanoseconds)");

    // load .lua script

    if(luaL_loadfile(L, path))
        bail(L, "luaL_loadfile() failed");

    if(lua_pcall(L, 0, 0, 0))
        bail(L, "lua_pcall() failed");

}

void lua_bridge_register_string(char *name, char *value) {
    ensure_initialized();
    lua_pushstring(L, value);
    lua_setglobal(L, name);
}

void lua_bridge_register_number(char *name, float value) {
    ensure_initialized();
    lua_pushnumber(L, value);
    lua_setglobal(L, name);
}

void lua_bridge_register_boolean(char *name, int value) {
    ensure_initialized();
    lua_pushboolean(L, value);
    lua_setglobal(L, name);
}

void lua_bridge_on_tick() {
    ensure_initialized();

    trading_decision = NOOP;

    lua_getglobal(L, "on_tick");
    if (lua_pcall(L, 0, 0, 0))
        bail(L, "lua_pcall() failed");
}

void lua_bridge_print_vars() {
    lua_getglobal(L, "booltest");
    lua_getglobal(L, "floattest");
    lua_getglobal(L, "inttest");

    printf("bool: %d\n", lua_toboolean(L, -3));

    double n = lua_tonumber(L, -2);
    if (n == (int)n) {
        printf("int: %d\n", (int)n);
    } else {
        printf("double: %f\n", n);
    }

    n = lua_tonumber(L, -1);
    if (n == (int)n) {
        printf("int: %d\n", (int)n);
    } else {
        printf("double: %f\n", n);
    }
}

void lua_bridge_teardown() {
    ensure_initialized();
    lua_close(L);
    L = 0;
}

void lua_bridge_open_config(char *path) {
    ensure_initialized();

    if(luaL_loadfile(L, path) || lua_pcall(L, 0, 0, 0)) {
        bail(L, "failed to load config file");
    }
}

char * lua_bridge_get_string_var(char *name) {
    ensure_initialized();

    lua_getglobal(L, name);

    if(!lua_isstring(L, -1)) {
        char err[255];
        sprintf(err, "'%s' should be a valid string name", name);
        bail(L, err);
    }

    return (char *)lua_tostring(L, -1);
}

int lua_bridge_get_int_var(char *name) {
    ensure_initialized();

    lua_getglobal(L, name);

    if(!lua_isnumber(L, -1)) {
        char err[255];
        sprintf(err, "'%s' should be a valid integer name", name);
        bail(L, err);
    }

    return (int)lua_tonumber(L, -1);
}

// ===== TABLE FUNCTIONS ===========================================================================

void lua_bridge_create_table(int size) {
    ensure_initialized();
    lua_createtable(L, size, 0);
}

void lua_bridge_push_table_integer(int num) {
    ensure_initialized();
    lua_pushinteger(L, num);
}

void lua_bridge_push_table_number(float num) {
    ensure_initialized();
    lua_pushnumber(L, num);
}

void lua_bridge_push_table_string(char *name) {
    ensure_initialized();
    lua_pushstring(L, name);
}

void lua_bridge_set_table(int offset) {
    ensure_initialized();
    lua_rawset(L, offset);
}

void lua_bridge_finalize_table(char *name) {
    ensure_initialized();
    lua_setglobal(L, name);
}
