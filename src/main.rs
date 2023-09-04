/* Git-Push-Extended
 *============================================
 * Purpose:     Provide a method to automate the mapping of branches during import
 * Notes:       This is meant to be part of a grater MuiltiSite tooling.
 */


use clap::ArgGroup;
use std::io::Read;
use std::fs::File;
use std::process::Command as cmd;
use std::error::Error;
use std::collections::HashMap;
use std::collections::HashSet;



use clap::{ command, Arg, ArgAction, ArgMatches, Command, ValueHint};

use rand::{distributions::Alphanumeric, Rng};

use substring::Substring;

use grep_searcher::SearcherBuilder;
use grep_searcher::sinks::UTF8;
use grep_regex::RegexMatcher;

use yaml_rust::YamlLoader;
use yaml_rust::Yaml;



fn main() 
{

/*
        .arg(Arg::new("verbose").help("Verbose messaging")
            .short('v').long("verbose")            
            .action(ArgAction::SetTrue)
            .global(true))
*/

    let cli = command!() // requires `cargo` feature
        .args_conflicts_with_subcommands(true)
        .subcommand_precedence_over_arg(true)
        .arg(Arg::new("remote_path").help("Run in a given path")
            .short('C')
            .value_hint(ValueHint::DirPath)            
            .global(true))
        .arg(Arg::new("force").help("Run using the force push")
            .short('f').long("force")
            .action(ArgAction::SetTrue)
            .global(true))
        .arg(Arg::new("prefix").help("Append Prefix to the branch nane, can include \"/\"")
            .short('p').long("prefix")
            .value_name("Pre-fix"))
        .arg(Arg::new("prefixWithDash").help("Append prefix and - to the branch name")
            .long("prefixWithDash")
            .value_name("Pre-fix"))
        .arg(Arg::new("postfix").help("Append Postfix to the branch name")
            .short('P').long("postfix")
            .value_name("Post-fix"))
        .arg(Arg::new("postfixWithDash").help("Append - and Postfix to the branch name")
            .long("postfixWithDash")
            .value_name("Post-fix"))
        .arg(Arg::new("include").help("Include branch by pattern or exact name; Comma(,) delineate")
            .short('i').long("include")
            .value_name("Exact/Pattern")
            .value_delimiter(','))
        .arg(Arg::new("exclude").help("Exclude branch by pattern or exact name; Comma(,) delineate")
            .short('e').long("exclude")
            .value_name("Exact/Pattern")
            .value_delimiter(','))
        .arg(Arg::new("importing").help("Select what git object to import Branches, Tags, or both")
            .short('I').long("Importing")
            .value_name("Type")
            .value_parser(["Tags", "Branches", "Both"]).default_value("Both"))
        .arg(Arg::new("url").help("Import Address/Location for the Mirror/Bare to import to")
            .required(true))
        .group(ArgGroup::new("prefix")
            .args(["prefix", "prefixWithDash"])
            .required(false))
        .group(ArgGroup::new("postfix")
            .args(["postfix", "postfixWithDash"])
            .required(false))
        .subcommand(Command::new("YAML").about("Allows to run a predefined rules")
            .arg(Arg::new("yaml").help("Yaml file")
                .value_hint(ValueHint::FilePath)
                .required(true))
            .arg(Arg::new("url").help("Import Address/Location for the Mirror/Bare to import to")
                .required(true))
        ).get_matches();

    


    if cli.get_flag("verbose") 
    {

        println!("C: {:?}", cli.get_one::<String>("remote_path"));
        println!("f: {:?}", cli.get_flag("force"));


        if let Some(sub) = cli.subcommand_matches("yaml") 
        {
            println!("Yaml: {:?}",   sub.get_one::<String>("yaml").unwrap());
            println!("Remote: {:?}", sub.get_one::<String>("url").unwrap());
        }
        else 
        {
            println!("Prefix {:?}",     cli.get_one::<String>("prefix").unwrap());
            println!("Postfix {:?}",    cli.get_one::<String>("postfix").unwrap());
            println!("Include: {:?}",   cli.get_one::<String>("include").unwrap());
            println!("Exclude: {:?}",   cli.get_one::<String>("exclude").unwrap());
            println!("Importing: {:?}", cli.get_one::<String>("importing").unwrap());
            println!("Remote: {:?}",    cli.get_one::<String>("url").unwrap());
        }
    }

   
    
    git_repo_test(&cli);
    let remote_name = get_remote(&cli);
    

    if cli.get_flag("verbose") 
    {println!("remote_name: {:?}", remote_name);}


    import(&cli, remote_name);    
}

/* git_repo_test
 *============================================
 * Purpose:     Test to see if repo is in good shape
 * Input:       CLi
 * Results:     None
 * Notes:       
 */
fn git_repo_test(cli: &ArgMatches) 
{
    
    let mut command = std::process::Command::new("git");

    if let Some(remote_path) = cli.get_one::<String>("remote_path") 
    {command.current_dir(remote_path);}

    let output = command.arg("rev-parse").arg("--show-toplevel")
        .output().expect("Command failed: git --show-toplevel");


    if !output.status.success() 
    {
        eprintln!("Push-Extended must be run in a git repository");
        std::process::exit(25);
    }


    command = std::process::Command::new("git");

    if let Some(remote_path) = cli.get_one::<String>("remote_path") 
    {command.current_dir(remote_path);}

    let output = command.arg("rev-parse")
        .arg("--is-bare-repository").output()
        .expect("Command failed: git rev-parse");


    if !output.status.success() 
    {
        eprintln!("Push-Extended must be run in a a bare repository");
        std::process::exit(25);
    }
}


/* get_remote
 *============================================
 * Purpose:     Either use existing set or set
 * Input:       CLI
 * Results:     Remote name
 * Notes:       
 */
fn get_remote(cli: &ArgMatches) -> String 
{

    let mut command = cmd::new("git");

    if let Some(remote_path) = cli.get_one::<String>("remote_path") 
    {command.current_dir(remote_path);}

    let output = command.arg("remote")
        .arg("-v").output().expect("Command failed: git remove -V");


    if !output.status.success() {
        eprintln!("Push-Extended local repository error");
        std::process::exit(20);
    }

    let given_url: String;

    if let Some(sub) = cli.subcommand_matches("yaml") 
    {given_url = sub.get_one::<String>("url").unwrap().to_string();}
    else 
    {given_url = cli.get_one::<String>("url").unwrap().to_string();}

    let stdout = String::from_utf8(output.stdout).unwrap(); 

    for (_i, value) in stdout.split('\n').enumerate() 
    {
        if value.to_string() != "" 
        {
            let mut url = value.to_string();
            
            url = url.substring(url.find("\t").unwrap()+1, url.len()).to_string();
            url = url.substring(0, url.find(" ").unwrap()).to_string();

            if url == given_url  
            {
                let mut name = value.to_string();
            
                name = name.substring(0, name.find("\t").unwrap()).to_string();
                return name
            }
        }
    }


    let remote_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric).take(16)
        .map(char::from).collect();


    command = cmd::new("git");

    if let Some(remote_path) = cli.get_one::<String>("remote_path") 
    {command.current_dir(remote_path);}

    let output = command.arg("remote").arg("add")
        .arg(remote_name.clone()).arg(given_url)
        .output().expect("Command failed: git remote add");


    if !output.status.success() 
    {
        eprintln!("Push-Extended remote repository error");
        std::process::exit(21);
    }

    return remote_name;
}


/* import
 *============================================
 * Purpose:     Import branches
 * Input:       Chsum, and remote name
 * Results:     NONE
 * Notes:       
 */
fn import(cli: &ArgMatches, remote_name: String) 
{
    let mut status: HashMap<String, String> = HashMap::new();
    let mut rules: Vec<HashMap<String, String>> = vec![];

    let importing = cli.get_one::<String>("importing").unwrap();

    if let Some(sub) = cli.subcommand_matches("yaml") 
    { 
        let doc = load_yaml(sub.get_one::<String>("yaml").unwrap());

        assert!(doc.len() == 1,    "Should only be one document length.\n");
        assert!(doc[0].is_array(), "Should be using array of selection\n");

      
        for rule in doc[0].as_vec().expect("Failed a to convert to vector") 
        {
            if !rule.is_null()
            {
                let rule_hash   = rule.as_hash().expect("Failed a to convert to hash");
                let mut final_hash: HashMap<String, String> = HashMap::new();

                
                let mut include = Yaml::Null;
                if rule_hash.contains_key(&Yaml::from_str("include"))
                {include = rule_hash.get(&Yaml::from_str("include")).unwrap().clone();}
                else if rule_hash.contains_key(&Yaml::from_str("Include"))
                {include = rule_hash.get(&Yaml::from_str("Include")).unwrap().clone();}
                else if rule_hash.contains_key(&Yaml::from_str("INCLUDE"))
                {include = rule_hash.get(&Yaml::from_str("INCLUDE")).unwrap().clone();}

                if include != Yaml::Null
                { 
                    let mut final_include: String = "".to_string();                
                    for include_rule in include.as_vec().unwrap()
                    {
                        if final_include != "" 
                        {final_include = final_include +",";}
                        final_include  = final_include + include_rule.as_str().unwrap();
                    }

                    if final_include != ""
                    { final_hash.insert("include".to_string(), final_include.to_string()); }
                }


                let mut exclude = Yaml::Null;
                if rule_hash.contains_key(&Yaml::from_str("exclude"))
                {exclude = rule_hash.get(&Yaml::from_str("exclude")).unwrap().clone();}
                else if rule_hash.contains_key(&Yaml::from_str("Exclude"))
                {exclude = rule_hash.get(&Yaml::from_str("Exclude")).unwrap().clone();}
                else if rule_hash.contains_key(&Yaml::from_str("EXCLUDE"))
                {exclude = rule_hash.get(&Yaml::from_str("EXCLUDE")).unwrap().clone();}

                

                if exclude != Yaml::Null
                {  
                    let mut final_exclude: String = "".to_string();        
                    for exclude_rule in exclude.as_vec().unwrap()
                    {
                        if final_exclude != "" 
                        {final_exclude = final_exclude + "," ;}
                        final_exclude = final_exclude + exclude_rule.as_str().unwrap();
                    }
                    

                    if final_exclude != ""
                    { final_hash.insert("exclude".to_string(), final_exclude.to_string()); }
                }


                let mut prefix  = "";
                if rule_hash.contains_key(&Yaml::from_str("prefix"))
                {prefix = rule_hash.get(&Yaml::from_str("prefix")).unwrap().as_str().unwrap();}
                else if rule_hash.contains_key(&Yaml::from_str("Prefix"))
                {prefix = rule_hash.get(&Yaml::from_str("Prefix")).unwrap().as_str().unwrap();}
                else if rule_hash.contains_key(&Yaml::from_str("PREFIX"))
                {prefix = rule_hash.get(&Yaml::from_str("PREFIX")).unwrap().as_str().unwrap();}

                if prefix != ""
                {final_hash.insert("prefix".to_string(), prefix.to_string());}


                let mut postfix = "";
                if rule_hash.contains_key(&Yaml::from_str("postfix"))
                {postfix = rule_hash.get(&Yaml::from_str("postfix")).unwrap().as_str().unwrap();}
                else if rule_hash.contains_key(&Yaml::from_str("Postfix"))
                {postfix = rule_hash.get(&Yaml::from_str("Postfix")).unwrap().as_str().unwrap();}
                else if rule_hash.contains_key(&Yaml::from_str("POSTFIX"))
                {postfix = rule_hash.get(&Yaml::from_str("POSTFIX")).unwrap().as_str().unwrap();}

                
                if postfix != ""
                { final_hash.insert("postfix".to_string(), postfix.to_string()); }

                
                rules.push(final_hash);
            }
        }
    }
    else if importing == "Both" || importing == "Branches" 
    {
        let mut final_hash: HashMap<String, String> = HashMap::new();

        let postfix = cli.get_one::<String>("postfix");
        if postfix != None   {final_hash.insert("postfix".to_string(), postfix.expect("Failed a to convert to string").to_string());}
        
        let postfix_with_dash = cli.get_one::<String>("postfixWithDash");
        if postfix_with_dash != None   {final_hash.insert("postfix".to_string(), "-".to_owned()+&postfix_with_dash.expect("Failed a to convert to string").to_string());}

        let prefix  = cli.get_one::<String>("prefix");
        if prefix != None   {final_hash.insert("prefix".to_string(), prefix.expect("Failed a to convert to string").to_string());}
        
        let prefix_with_dash = cli.get_one::<String>("prefixWithDash");
        if prefix_with_dash != None  {final_hash.insert("prefix".to_string(), "-".to_owned()+&prefix_with_dash.expect("Failed a to convert to string").to_string());}

        let include = cli.get_one::<String>("include");
        if include != None   {final_hash.insert("include".to_string(), include.expect("Failed a to convert to string").to_string());}
        
        let exclude = cli.get_one::<String>("exclude");
        if exclude != None   {final_hash.insert("exclude".to_string(), exclude.expect("Failed a to convert to string").to_string());}
        
        rules.push(final_hash);
    }

    if rules.len() == 1 &&
        !rules[0].contains_key("prefix")  && !rules[0].contains_key("postfix") &&
        !rules[0].contains_key("include") && !rules[0].contains_key("exclude")
    {
        let mut command = cmd::new("git");
        command.arg("push").arg("--all");

        if cli.get_flag("force") {command.arg("--force");}

        command.arg(remote_name.clone());

        if let Some(remote_path) =  cli.get_one::<String>("remote_path")
        {command.current_dir(remote_path);}

        let output = command.output().expect("Command failed: git push --all");
        if !output.status.success() 
        {
            eprintln!("Push-Extended remote: git push --all, error");
            std::process::exit(20);
        }
    }
    else 
    {
        let mut branches = get_branches(cli);
        
        for rule in rules
        {
            println!("Processing Rule:");
            if rule.contains_key("exclude")
            {println!("   Exclude: {:?}", rule.get("exclude"));}
            else {println!("   Exclude: N/A");}
            if rule.contains_key("include")
            {println!("   Include: {:?}", rule.get("include"));}
            else {println!("   Include: N/A");}
            if rule.contains_key("prefix")
            {println!("   Prefix:  {:?}", rule.get("prefix"));}
            else {println!("   Prefix: N/A");}
            if rule.contains_key("postfix")
            {println!("   Postfix: {:?}\n", rule.get("postfix"));}
            else {println!("   Postfix: N/A\n");}

            let mut branches_vec = convert_to_vec(&branches);
        
            if rule.contains_key("exclude")
            {
                for value in rule.get("exclude").expect("Failed to split").split(",")
                {
                    if value != ""
                    {
                        let mut sink: Vec<u8> = vec![];

                        let _ = grep(&branches_vec, value, true, &mut sink);
                        branches_vec = sink
                    }
                }                        
            }

            if branches_vec.len() == 0 {continue;}

            if rule.contains_key("include")         
            {
                let mut grouping: Vec<u8> = vec![];
                for value in rule.get("include").expect("Failed to split").split(",")
                {
                    println!("{:?}", value);
                    if value != ""
                    {
                        let mut sink: Vec<u8> = vec![];

                        let _ = grep(&branches_vec, value, false, &mut sink);
                        for byte in sink  {grouping.push(byte);}
                    }
                }

                branches_vec = grouping;
            }

            if branches_vec.len() == 0 {continue;}
            for (_size, mut branch) in str::replace(std::str::from_utf8(&branches_vec).unwrap(), "\n\n", "\n").split("\n").enumerate() 
            {
                branch = branch.trim();
                
                if branch == "" 
                {continue}

                
                let mut final_branch = String::from(branch);

                if rule.contains_key("prefix") 
                {final_branch.insert_str(0, rule.get("prefix").expect("Failed to convert String").as_str());}
                if rule.contains_key("postfix")
                {final_branch.push_str(rule.get("postfix").expect("Failed to convert String").as_str());}


                println!("Importing: {:?} -> {:?}", branch, final_branch);


                let mut command = cmd::new("git");
                command.arg("push");

                if cli.get_flag("force") {command.arg("--force");}
                
                command.arg(remote_name.clone()).arg(branch.to_owned() + ":"+&final_branch);
                    

                if let Some(remote_path) =  cli.get_one::<String>("remote_path")
                {command.current_dir(remote_path);}

                let output = command.output().expect("Command failed: git push branch");

                println!("{:?}", command);

                if output.status.success() 
                {status.insert("Successfully, Push Branch".to_string(), "{branch} -> {final_branch}".to_string());}
                else
                {status.insert("Failed, Push Branch".to_string(), "{branch} -> {final_branch}".to_string());}

                branches.remove(branch);
            }
        }
    }
    

    if importing == "Both" || importing == "Tags" 
    {
        let mut command = cmd::new("git");

        if let Some(remote_path) =  cli.get_one::<String>("remote_path")
        {command.current_dir(remote_path);}

        command.arg("push").arg("--tags");

        if cli.get_flag("force")   
        {command.arg("--force"); }

                
        let output = command.arg(remote_name.clone())
            .output().expect("Command failed: git push --tags");


        if !output.status.success() 
        {
            eprintln!("Push-Extended: remote push tags error");
            std::process::exit(19);
        }
    }    
}


/* convert_to_vec
 *============================================
 * Purpose:     convert the branches to vector to be search
 * Input:       HashSet of content
 * Results:     Vector of content
 * Notes:       Add \n as a deliminator
 */
fn convert_to_vec(content: &HashSet<String>) ->  Vec<u8>
{
    let mut list:Vec<u8> = vec![];

    for branch in content 
    {
        for value in branch.as_bytes()
        {list.push(*value);}
        list.push(10u8);
    }

    return list;
}


/* get_branches
 *============================================
 * Purpose:     Get the branch as vector
 * Input:       Cli
 * Results:     List of branches
 * Notes:       Need to filter off "* ", "  "
 */
fn get_branches(cli: &ArgMatches) -> HashSet<String>
{
    let mut branches:HashSet<String> = HashSet::new();
    let mut binding = cmd::new("git");
    let command = binding.arg("branch");
        

    if let Some(remote_path) =  cli.get_one::<String>("remote_path")
    {command.current_dir(remote_path);}

    let output = command.output().expect("Command failed: git branch");
    
    if output.status.success()
    {
        let mut back:u8 = 0;
        let mut bucket:Vec<u8> = vec![];

        for value in output.stdout 
        {
            if value == 32u8 
            {
                if back != 0 {back = 0;}  //remove 32, 32 and 42, 32
                else         {back = 32;} //store 32
            }
            else if value == 42u8
            {
                if back != 0 //not expecting 42, 42 or 32 42
                {
                    bucket.push(back);
                    bucket.push(42u8);

                    back = 0;
                }
                else {back = 42;} //store 42
            }
            else if value == 10u8
            {
                branches.insert(String::from_utf8(bucket).expect("Failed to convert to string, $bucket").into());

                bucket = vec![];
            }
            else 
            {
                if back != 0  //not expect 42, !32
                {
                    bucket.push(back);
                    back = 0;
                }

                bucket.push(value);         
            }
        }

        if back != 0 {bucket.push(back);}

        if bucket.len() > 0
        {branches.insert(String::from_utf8(bucket).expect("Failed to convert to string, $bucket").into());}
    }
    return branches.clone()
}


/* grep
 *============================================
 * Purpose:     Perform a Linux mimic grep
 * Input:       Content, Matching, is this an excluded, final sets
 * Results:     Results and errors
 * Notes:       This is based on another code, striped down. May not need to return error
 */
fn grep(content: &[u8], matching: &str, exclude: bool, sink: &mut Vec<u8>) -> Result<(), Box<dyn Error>>
{
    let matcher     = RegexMatcher::new(matching)?;

    let mut searcher = SearcherBuilder::new();
    searcher.invert_match(exclude);
    searcher.build().search_slice(&matcher, content, UTF8(|_lnum, line| {

        for i in line.as_bytes() {sink.push(i.clone());}
        sink.push(10);
        
        Ok(true)
    }))?;

    return Ok(())
}



/* load_yaml
 *============================================
 * Purpose:     Load the yaml from file
 * Input:       File Path
 * Results:     Vec of Yamls
 * Notes:       
 */
fn load_yaml(file: &str) -> Vec<Yaml> {
    let mut file = File::open(file).expect("Unable to open File $file");
    let mut contents = String::new();

    file.read_to_string(&mut contents).expect("Unable to read $file");

    return YamlLoader::load_from_str(&contents).unwrap();
}