from pathlib import Path
import json,re

root=Path(__file__).resolve().parents[1]
def validate(item,schema,path="$"):
 if "const" in schema and item!=schema["const"]:raise AssertionError(path+":const")
 if "enum" in schema and item not in schema["enum"]:raise AssertionError(path+":enum")
 if "type" in schema:
  types=schema["type"] if isinstance(schema["type"],list) else [schema["type"]]
  handlers={"object":lambda v:isinstance(v,dict),"array":lambda v:isinstance(v,list),"integer":lambda v:isinstance(v,int) and not isinstance(v,bool),"boolean":lambda v:isinstance(v,bool),"string":lambda v:isinstance(v,str),"null":lambda v:v is None}
  if not any(handlers[t](item) for t in types):raise AssertionError(path+":type")
 if isinstance(item,dict):
  keys=schema.get("properties",{})
  if any(key not in item for key in schema.get("required",[])):raise AssertionError(path+":required")
  if schema.get("additionalProperties") is False and set(item)-set(keys):raise AssertionError(path+":extra")
  for key,value in item.items():
   if key in keys:validate(value,keys[key],path+"."+key)
 if isinstance(item,list):
  if len(item)<schema.get("minItems",0):raise AssertionError(path+":minItems")
  for index,value in enumerate(item):
   if "items" in schema:validate(value,schema["items"],f"{path}[{index}]")
 if isinstance(item,str):
  if len(item)<schema.get("minLength",0):raise AssertionError(path+":minLength")
  if "pattern" in schema and not re.fullmatch(schema["pattern"],item):raise AssertionError(path+":pattern")
 if isinstance(item,int) and not isinstance(item,bool):
  if item<schema.get("minimum",item) or item>schema.get("maximum",item):raise AssertionError(path+":range")

pairs=[
 ("economic-opportunity.schema.json","economic.opportunity.json"),
 ("economic-policy.schema.json","economic.policy.json"),
 ("economic-entry.schema.json","economic.entry.json"),
 ("economic-schedule.schema.json","economic.schedule.json"),
 ("economic-income-report.schema.json","economic.report.json"),
]
for schema_file,fixture_file in pairs:
 schema=json.loads((root/"contracts"/schema_file).read_text(encoding="utf-8"))
 fixture=json.loads((root/"contracts/fixtures"/fixture_file).read_text(encoding="utf-8"))
 validate(fixture,schema)
assert json.loads((root/"contracts/fixtures/economic.report.json").read_text(encoding="utf-8"))["accrued_profit"]==8500
print("F10_CONTRACTS_OK fixtures=5 monetary_unit=BRL_cents")
