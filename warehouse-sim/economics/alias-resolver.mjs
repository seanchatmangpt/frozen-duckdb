// Node ESM resolve hook: maps the Next.js "@/..." tsconfig alias to real file
// URLs under the warehouse-sim root so plain `node` (type-stripping) can load
// the sim's REAL generated route modules from economics/scorecard.ts.
// Registered from run_scorecard.sh via --import. Lane S5.
const ROOT = new URL("../", import.meta.url); // warehouse-sim/
export async function resolve(specifier, context, nextResolve) {
  if (specifier.startsWith("@/")) {
    const target = new URL(specifier.slice(2) + ".ts", ROOT);
    return { url: target.href, shortCircuit: true };
  }
  return nextResolve(specifier, context);
}
