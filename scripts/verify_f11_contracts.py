from pathlib import Path
import json,re

root=Path(__file__).resolve().parents[1]
def verify(value,schema,path="$"):
    if "const" in schema and value!=schema["const"]:raise AssertionError(path+":const")
    if "enum" in schema and value not in schema["enum"]:raise AssertionError(path+":enum")
    if "type" in schema:
        handlers={"object":lambda v:isinstance(v,dict),"array":lambda v:isinstance(v,list),"integer":lambda v:isinstance(v,int) and not isinstance(v,bool),"boolean":lambda v:isinstance(v,bool),"string":lambda v:isinstance(v,str)}
        if not handlers[schema["type"]](value):raise AssertionError(path+":type")
    if isinstance(value,dict):
        keys=schema.get("properties",{})
        if any(k not in value for k in schema.get("required",[])):raise AssertionError(path+":required")
        if schema.get("additionalProperties") is False and set(value)-set(keys):raise AssertionError(path+":extra")
        for k,item in value.items():
            if k in keys:verify(item,keys[k],path+"."+k)
    if isinstance(value,list):
        for item in value:
            if "items" in schema:verify(item,schema["items"],path+"[]")
    if isinstance(value,int) and not isinstance(value,bool):
        if value<schema.get("minimum",value) or value>schema.get("maximum",value):raise AssertionError(path+":range")
    if isinstance(value,str) and "pattern" in schema and not re.fullmatch(schema["pattern"],value):raise AssertionError(path+":pattern")
pairs=[("availability-health.schema.json","availability.health.json"),("availability-matrix.schema.json","availability.matrix.json")]
for schema,fixture in pairs:
    verify(json.loads((root/"contracts/fixtures"/fixture).read_text(encoding="utf-8")),
        json.loads((root/"contracts"/schema).read_text(encoding="utf-8")))
print("F11_CONTRACTS_OK fixtures=2 tiers=1,4,8,16")
