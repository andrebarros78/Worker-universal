from pathlib import Path
import json,re

root=Path(__file__).resolve().parents[1]
def validate(value,schema,path="$"):
    if "const" in schema and value!=schema["const"]:raise AssertionError(path+":const")
    if "enum" in schema and value not in schema["enum"]:raise AssertionError(path+":enum")
    if "type" in schema:
        funcs={"object":lambda x:isinstance(x,dict),"array":lambda x:isinstance(x,list),
               "integer":lambda x:isinstance(x,int) and not isinstance(x,bool),
               "string":lambda x:isinstance(x,str),"boolean":lambda x:isinstance(x,bool)}
        if not funcs[schema["type"]](value):raise AssertionError(path+":type")
    if isinstance(value,dict):
        props=schema.get("properties",{})
        if any(k not in value for k in schema.get("required",[])):raise AssertionError(path+":required")
        if schema.get("additionalProperties") is False and set(value)-set(props):
            raise AssertionError(path+":extra")
        for k,item in value.items():
            if k in props:validate(item,props[k],path+"."+k)
    if isinstance(value,list):
        for item in value:
            if "items" in schema:validate(item,schema["items"],path+"[]")
    if isinstance(value,str):
        if len(value)<schema.get("minLength",0):raise AssertionError(path+":minLength")
        if "pattern" in schema and not re.fullmatch(schema["pattern"],value):
            raise AssertionError(path+":pattern")
    if isinstance(value,int) and not isinstance(value,bool):
        if value<schema.get("minimum",value):raise AssertionError(path+":range")
pairs=[
 ("f12-secret-policy.schema.json","f12.secret-policy.json"),
 ("f12-backup-manifest.schema.json","f12.backup-manifest.json"),
 ("f12-audit-event.schema.json","f12.audit-event.json"),
 ("f12-release-inventory.schema.json","f12.release-inventory.json"),
]
for schema_file,fixture_file in pairs:
    schema=json.loads((root/"contracts"/schema_file).read_text(encoding="utf-8"))
    fixture=json.loads((root/"contracts/fixtures"/fixture_file).read_text(encoding="utf-8"))
    validate(fixture,schema)
print("F12_CONTRACTS_OK fixtures=4")
