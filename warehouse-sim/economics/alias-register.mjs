// Registers alias-resolver.mjs for `node --import`. Lane S5.
import { register } from "node:module";
register("./alias-resolver.mjs", import.meta.url);
