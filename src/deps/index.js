"use strict";

function safeUseClass(clazz) {
  try {
    return Java.use(clazz);
  } catch (_e) {
    console.error(`error: "${clazz}" not found`);
    return null;
  }
}

function printBacktrace() {
  console.log(
    Java.use("android.util.Log").getStackTraceString(
      Java.use("java.lang.Exception").$new()
    )
  );
}

function monitorEntry(obj) {
  const { clazzVm, methodName, backtrace, path, injectArgs, injectRet } = obj;
  clazzVm[methodName].overloads.forEach((m) => {
    m.implementation = function () {
      let args = arguments;
      let msg = Array.from(args).join(", ");
      if (injectArgs) {
        args = injectArgs(args);
        msg += " #=># " + args.join(",");
      }

      console.log(`enter [${path}]: ${msg}`);
      if (backtrace) {
        printBacktrace();
      }

      let ret = this[methodName].apply(this, args);

      let msg2 = `${ret}`;
      if (injectRet) {
        ret = injectRet(ret);
        msg2 += " #=># " + ret;
      }

      console.log(`exit [${path}]: ${msg2}`);
      return ret;
    };
  });
}

/// monitor a single method
function method(m, obj = {}) {
  const pos = m.lastIndexOf(".");
  const clazz = m.slice(0, pos);
  const methodName = m.slice(pos + 1);
  let clazzVm = safeUseClass(clazz);
  if (!clazzVm) {
    return;
  }
  const { backtrace, injectArgs, injectRet } = obj;
  monitorEntry({
    clazzVm,
    methodName,
    backtrace,
    path: m,
    injectArgs,
    injectRet,
  });
}

/// monitor all methods in class c
function clazz(c, obj = {}) {
  let clazzVm = safeUseClass(c);
  if (!clazzVm) {
    return;
  }

  const { backtrace, injectArgs, injectRet } = obj;
  clazzVm.class.getDeclaredMethods().forEach((m) => {
    monitorEntry({
      clazzVm,
      methodName: m.getName(),
      backtrace,
      path: `${clazzVm}.${m.getName()}`,
      injectArgs: null,
      injectRet: null,
    });
  });
}

function main() {
  console.log("++ frida script is loaded ++");
  Java.perform(() => {
    console.log("vm is attached");
  });
}

main();
