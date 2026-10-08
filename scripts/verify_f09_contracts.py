from pathlib import Path
import json,re

root=Path(__file__).resolve().parents[1]

def validate(item, schema, path="$"):
 if "const" in schema and item!=schema["const"]:
  raise AssertionError(path+":const")
 if "enum" in schema and item not in schema["enum"]:
  raise AssertionError(path+":enum")
 if "type" in schema:
  choices=schema["type"] if isinstance(schema["type"],list) else [schema["type"]]
  check={"object":lambda a:isinstance(a,dict),"array":lambda a:isinstance(a,list),"string":lambda a:isinstance(a,str),"integer":lambda a:isinstance(a,int) and not isinstance(a,bool),"boolean":lambda a:isinstance(a,bool),"null":lambda a:a is None}
  if not any(check[t](item) for t in choices):
   raise AssertionError(path+":type")
 if isinstance(item,dict):
  props=schema.get("properties",{})
  for k in schema.get("required",[]):
   if k not in item:raise AssertionError(path+":missing:"+k)
  if schema.get("additionalProperties") is False and set(item)-set(props):
   raise AssertionError(path+":unexpected:"+str(set(item)-set(props)))
  for k,v in item.items():
   if k in props:validate(v,props[k],path+"."+k)
 if isinstance(item,list):
  if len(item)<schema.get("minItems",0):
   raise AssertionError(path+":minItems")
  for i,v in enumerate(item):
   if "items" in schema:validate(v,schema["items"],f"{path}[{i}]")
 if isinstance(item,str):
  if len(item)<schema.get("minLength",0):raise AssertionError(path+":minLength")
  if "pattern" in schema and not re.fullmatch(schema["pattern"],item):raise AssertionError(path+":pattern")
 if isinstance(item,int) and not isinstance(item,bool):
  if item<schema.get("minimum",item):raise AssertionError(path+":min")
pairs=[
 ("platform-adapter-config.schema.json","platform.adapter.config.json"),
 ("platform-adapter-plan.schema.json","platform.adapter.plan.json"),
 ("platform-adapter-outcome.schema.json","platform.adapter.outcome.json"),
]
for schema,fixture in pairs:
 s=json.loads((root/"contracts"/schema).read_text(encoding="utf-8"))
 value=json.loads((root/"contracts/fixtures"/fixture).read_text(encoding="utf-8"))
 validate(value,s)

ids=["generic_web","generic_form","generic_invoice","homeocta","99freelas"]
schema=json.loads((root/"contracts/adapter-manifest.schema.json").read_text(encoding="utf-8"))
for id in ids:
 value=json.loads((root/"contracts/fixtures"/f"platform.manifest.{id}.json").read_text(encoding="utf-8"))
 validate(value,schema)
 assert value["adapter_id"]==id
 assert value["simulation_supported"] is True
print("F09_CONTRACTS_OK fixtures=8 manifests=5 platform_schemas=3")
