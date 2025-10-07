package main

import (
	"fmt"
	"log"
	"os"

	"kcura_go/ffi"
)

func main() {
	fmt.Println("Starting Go KCura smoke test...")

	// Initialize KCura with in-memory database
	kc := ffi.New(":memory:")
	defer kc.Free()
	fmt.Println("✓ KCura initialized")

	// Test OWL conversion
	owlContent := `@prefix ex: <http://example.org/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

ex:Person a owl:Class .
ex:name a owl:DatatypeProperty ;
    rdfs:domain ex:Person ;
    rdfs:range xsd:string .`

	err := kc.Convert(owlContent, "", `{}`)
	if err != nil {
		log.Fatalf("❌ OWL conversion failed: %v", err)
	}
	fmt.Println("✓ OWL conversion completed")

	// Test SPARQL query
	query := "SELECT ?s WHERE { ?s a ex:Person }"
	result, err := kc.Query(query)
	if err != nil {
		log.Fatalf("❌ SPARQL query failed: %v", err)
	}
	fmt.Printf("✓ SPARQL query executed successfully\n")
	fmt.Printf("  Result: %d rows\n", len(result.Rows))

	// Test SHACL validation
	shaclContent := `@prefix ex: <http://example.org/> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [
        sh:path ex:name ;
        sh:datatype xsd:string ;
        sh:minCount 1
    ] .`

	validation, err := kc.Validate(shaclContent)
	if err != nil {
		log.Fatalf("❌ SHACL validation failed: %v", err)
	}
	fmt.Printf("✓ SHACL validation completed\n")
	fmt.Printf("  Violations: %d\n", len(validation.Violations))

	// Test hook registration
	hookSpec := `{
		"id": "test-guard",
		"kind": "guard",
		"predicate_lang": "sql",
		"predicate": "SELECT 1 WHERE 1=0",
		"action_kind": "sql",
		"action": "SELECT 1"
	}`

	err = kc.RegisterHook(hookSpec)
	if err != nil {
		log.Fatalf("❌ Hook registration failed: %v", err)
	}
	fmt.Println("✓ Hook registered successfully")

	// Test hook execution
	err = kc.Guards()
	if err != nil {
		log.Fatalf("❌ Guard hooks execution failed: %v", err)
	}
	fmt.Println("✓ Guard hooks executed")

	// Test database info
	info, err := kc.Info()
	if err != nil {
		log.Fatalf("❌ Database info failed: %v", err)
	}
	fmt.Printf("✓ Database info retrieved: %s\n", info.Status)

	fmt.Println("\n🎉 All Go smoke tests passed!")
	os.Exit(0)
}
