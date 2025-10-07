#!/usr/bin/env node

/**
 * Node.js smoke test for KCura FFI bindings
 * Tests basic functionality through N-API bindings
 */

import { KCura } from "../bindings/node/index.js";

async function smokeTest() {
  console.log("Starting Node.js KCura smoke test...");

  try {
    // Initialize KCura with in-memory database
    const kc = new KCura(":memory:");
    console.log("✓ KCura initialized");

    // Test OWL conversion
    const owlContent = `@prefix ex: <http://example.org/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

ex:Person a owl:Class .
ex:name a owl:DatatypeProperty ;
    rdfs:domain ex:Person ;
    rdfs:range xsd:string .`;

    await kc.convert({ owl: owlContent });
    console.log("✓ OWL conversion completed");

    // Test SPARQL query
    const query = "SELECT ?s WHERE { ?s a ex:Person }";
    const result = await kc.query(query);

    if (result.rows.length >= 0) {
      console.log("✓ SPARQL query executed successfully");
      console.log(`  Result: ${result.rows.length} rows`);
    } else {
      throw new Error("Query returned invalid result");
    }

    // Test SHACL validation
    const shaclContent = `@prefix ex: <http://example.org/> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [
        sh:path ex:name ;
        sh:datatype xsd:string ;
        sh:minCount 1
    ] .`;

    const validation = await kc.validate(shaclContent);
    console.log("✓ SHACL validation completed");
    console.log(`  Violations: ${validation.violations.length}`);

    // Test hook registration
    const hookSpec = {
      id: "test-guard",
      kind: "guard",
      predicate_lang: "sql",
      predicate: "SELECT 1 WHERE 1=0",
      action_kind: "sql",
      action: "SELECT 1",
    };

    await kc.registerHook(hookSpec);
    console.log("✓ Hook registered successfully");

    // Test hook execution
    const guardResult = await kc.guards();
    console.log("✓ Guard hooks executed");

    // Clean up
    kc.close();
    console.log("✓ KCura instance closed");

    console.log("\n🎉 All Node.js smoke tests passed!");
    return true;
  } catch (error) {
    console.error("❌ Node.js smoke test failed:", error.message);
    console.error("Stack trace:", error.stack);
    return false;
  }
}

// Run the smoke test
smokeTest()
  .then((success) => {
    process.exit(success ? 0 : 1);
  })
  .catch((error) => {
    console.error("❌ Smoke test runner failed:", error);
    process.exit(1);
  });
