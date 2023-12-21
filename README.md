# Git-Push-Extended 
## Warning
>this is my first application in Rust. Further updates to clean up the code will eventually be done, when I become more experiences. I am open to making changes that improve the code format.

>This is meant to the part of a MultiSite system. So some of the implementation does not make sense in a standalone perspective.

---

## Purpose
>To provide an abilty to merge a remote site repoistory with a local site.


---
## Running
>### Standard
>>git-push-extended [-C \<remote_path>] [--force] [--prefix | --prefixWithDash \<Pre-Fix>] 
>>>[--postfix | --postfixWtihDash <Post-fix>] [--include \<pattern>,...]  
[--exclude \<pattern>,...] [--Importing \<Tags | Branches | Both>] <Repository>

>>Args:
>>>Repository: Can be a Git URL or Directory endpoints

>>Options:
>>>-C:  Directory to run in, should be a git Bear/Mirror repository  
-f | --force: run the git push command with -force flage    
-p | --prefix: Applies a prefix to branches, "/" are allowded but not "-"  
--prefixWithDash: Applies a prefix to branches, "/" are allowded but not "-"
-P | --postfix: Applies a postfix to branches, "/" are allowded but not "-"  
--postfixWithDash: Applies a postfix to branches, "/" are allowded but not "-"
-i | --include: Imports any branches that match pattern  
-e | --exclude: Imports any branches that does not match pattern  
--Importing: Allow you to select what to import type tags or branches or both, default to both


>### YAML
>>git-push-extended [-C \<remote_path>] [--force] YAML \<Yaml File> \<Repository>  

>>Args:
>>>Repository: Can be a Git URL or Directory endpoints \  
Yaml File: Path to a predefined Yaml File (See bellow for format)

>>Options:
>>>-C:  Directory to run in, should be a git Bear/Mirror repository \
-f | --force: run the git push command with -force flage

---
## YAML Format
>The contents must be an list of rules. The rules are applied in topdown order. Any branch that matches a rule will have any prefix/postfix applied, and dropped from the branch list for future rules.

>### Rules
>>Prefix:  Applies a prefix to branches, "/" and "-" are allowded  
Postfix:   Applies a postfix to branches, "/" and "-" are allowded  
Include:   Should be a list of patterns, if any pattern matches a branch that branch is included
Exclude:   Should be a list of patterns, if any pattern matches a branch that branch is excluded

```
- include:
  - <pattern>
  exclude:
  - <pattern>
  prefix: <string>
  postfix: <string>
```

## Example
> You have two sites, New York(NY) and Denver(DEN). You are exporting a mirror repo from NY to DEN. You want to import the branches Operations and Development to identical branches in DEN. The Topic branches from NY should recieve a prefix of "NY/". Finnally you do not want to import any branches with the prefix of "DEN/".

|NY | Import | DEN|
|---|---|---|
|Operation|Yes|Operation|
|Development|Yes|Development|
|Topic1|Yes|NY/Topic1|
|Topic3|Yes|NY/Topic2|
|DEN/Topic3|No|Topic3|

>### Standared
>> git-push-extended --include Operation,Development git@repo  
git-push-extended --prefix NY/ --exclude Operation,Development,DEN/* git@repo  


>### YAML
>> Yaml File, Input.yaml:
```
- Include:
  - Operation 
  - Development
- Exclude:
  - DEN*
  Prefix: NY/
```
>>call: git-push-extended YAML input.yaml git@repo  