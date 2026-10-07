use std::{collections::HashMap, env::self, io::{Read, Write}, path::{Path, PathBuf}, process::ExitCode};
use syn;
use serde_json;
use serde::{Deserialize,Serialize};
const TEMPL : &str = include_str!("templ.htm");
fn main()->ExitCode {
    let mod_list = get_crate_dirs("windows");
    if mod_list.is_empty(){
        eprintln!("not found windows crate");
        return ExitCode::from(1);
    }
    let args = env::args();
    match args.len(){
        2 =>{
            if let Ok(i) = args.into_iter().nth(1).unwrap().parse::<usize>(){
                if let Some(selected) = mod_list.get(i){
                    println!("read source from {}",selected.0.to_string_lossy().to_string());
                    if let Some(root) = get_mod_from_source(&selected.0.join("src/Windows")){
                        let mut s = build_json(&root);
                        s.version = selected.1.clone();
                        let out_name_ = selected.0.file_name().unwrap().to_string_lossy().to_string();
                        let out_name_s = out_name_ + ".json";
                        let out_name = Path::new(&out_name_s);

                        println!("write result to file {}", out_name_s);
                        let json_str = serde_json::to_string(&s).unwrap_or_default();
                        if let Ok(mut out) = std::fs::File::create(out_name){
                            let _ = out.write(json_str.as_bytes());
                            println!("write {} success", out_name_s);
                        }else{
                            eprintln!("create file {} fail",out_name_s);
                            return ExitCode::from(4);
                        }
                        let out_html_s =  "index.html";
                        println!("write ui file {}", out_html_s);
                        if let Ok(mut out_html) = std::fs::File::create(out_html_s){
                            let _ = out_html.write(TEMPL.replace("{{json}}", &json_str).as_bytes());
                            println!("write {} success", out_name_s);
                        }else{
                            eprintln!("create file {} fail", out_html_s);
                            return ExitCode::from(5);
                        }
                    }else{
                        eprintln!("parse crate fail");
                        return ExitCode::from(3);
                    }
                }else{
                    eprintln!("select crate fail: {}",i);
                }
            }else{
                eprintln!("not a valid number");
                return ExitCode::from(3);
            }
        },
        _=>{
            eprintln!("use [{} index] select which version to build",
                env::current_exe().unwrap().file_name().unwrap().to_string_lossy().to_string());
            return ExitCode::from(2);
        }
    }
    ExitCode::from(0)
}
fn build_json(root : &ModNode)->WinFn{
    let mut words_table : HashMap<String,usize> = HashMap::new();
    let mut words_vec = vec![];
    let mut ret = WinFn{version : String::default(), table : vec![], func : vec![]};
    let mut get_word_id = |w : &str|->usize{
        if let Some(id) = &words_table.get(w){
            return **id;
        }else{
            let cur_id = words_vec.len();
            let _ = &words_table.insert(w.to_owned(), words_vec.len());
            words_vec.push(w.to_owned());
            return cur_id;
        }
    };
    let mut proc = |f : &Func,parent : &Vec<&ModNode>|{
        let cur = FuncInfo{name : f.name.clone(),
            space : build_mod_space(&parent)
                .iter()
                .map(|s|{
                   get_word_id(&s)
                })
                .collect(),
            feature : [build_mod_feature(&parent),f.feature.clone()].concat()
                .iter()
                .map(|s|{
                    get_word_id(s)
                })
                .collect()
        };
        ret.func.push(cur);
    };
    walk_root(root,&vec![], &mut proc, true);
    ret.table = words_vec;
    return ret;
}

#[derive(Serialize, Deserialize)]
struct WinFn{
    version : String,
    table : Vec<String>,
    func : Vec<FuncInfo>,
}
#[derive(Serialize, Deserialize)]
struct FuncInfo{
    name : String,
    space : Vec<usize>,
    feature : Vec<usize>
}
fn read_mod_file_from_directory(d : &Path)->Option<String>{
    let mut ret = String::new();
    let _ = std::fs::File::open(d.join("mod.rs")).ok()?.read_to_string(&mut ret).ok()?;
    Some(ret)
}
enum Key{
    Func(String),
    Mod(String),
}

struct Func{
    feature :Vec<String>,
    name : String,
    sig : String,
}
struct ModNode{
    name : String,
    features : Vec<String>,
    function : Vec<Func>,
    children : Vec<ModNode>
}
fn proc_feature(m : &syn::Meta, out : &mut Vec<String>){
    match m{
        syn::Meta::Path(_)=>{

        },
        syn::Meta::NameValue(nv)=>{
            if nv.path.segments.last().and_then(|k|{
                k.ident.eq("feature").then_some(())
            }).is_some(){
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    out.push(s.value())
                }
            }
        },
        syn::Meta::List(lst)=>{
            if let Ok(l) = lst.parse_args_with(syn::punctuated::Punctuated::<syn::Meta,syn::Token![,]>::parse_terminated){
                for l1 in l{
                    proc_feature(&l1, out);
                }
            }

        }
    }
}
fn get_mod_from_source<'a>(src : &Path)->Option<ModNode>{
    let mut mod_root = ModNode{name : String::default(), features : vec![], function : vec![], children : vec![]};
    //println!("read file {}",&src.join("mod.rs").to_string_lossy().to_string());
    if let Ok(source) = std::fs::read_to_string(&src.join("mod.rs").to_string_lossy().to_string()){
    if let Ok(r) = syn::parse_file(&source){
        for node in r.items{
            match node{
                syn::Item::Mod(m)=>{
                    let mod_name = m.ident.to_string();
                    let mut out = vec![];
                    if let syn::Visibility::Public(_) = m.vis{
                        for m1 in m.attrs{
                            if m1.path().is_ident("cfg"){
                                proc_feature(&m1.meta, &mut out);
                            }
                        }
                    }
                    if !out.is_empty(){
                        //println!("mod: {}=>{}",mod_name,out.join("|"));
                    }
                    if let Some(mut child)  = get_mod_from_source(&src.join(&mod_name)){
                        child.name = mod_name;
                        child.features = out;
                        mod_root.children.push(child);
                    }
                },
                syn::Item::Fn(f)=>{
                    //println!("get fn: {}", f.sig.ident.to_string());
                    //let sig = quote! {#f.sig.inputs}.to_string();
                    if let syn::Visibility::Public(_) = f.vis{
                        if let syn::Safety::Unsafe(_) = f.sig.safety{
                            let fn_name = f.sig.ident.to_string();
                            let mut out = vec![];
                            for m in f.attrs{
                                proc_feature(&m.meta, &mut out);
                            }

                            //println!("fn: {}=>{}",fn_name,out.join("|"));
                            mod_root.function.push(Func{feature : out, name: fn_name, sig : String::default()});
                        }
                    }
                }
                _=>{

                }
            }
        }
    }
        return Some(mod_root);
    }
    None
}
fn get_cargo_registry_path()->String{
    std::env::var("USERPROFILE")
        .and_then(|s|{
            Ok(format!("{}/.cargo/registry/src",s).to_string())
        }).unwrap_or_default()
}
fn get_crate_dirs(name : &str)->Vec<(PathBuf,String)>{
    let index_str = regex::Regex::new(r##"\Aindex\.crates\.io\-.+\z"##).unwrap();
    let crate_str = regex::Regex::new(&format!(r##"\A({}\-\d+\.\d+\.\d+)\z"##,name)).unwrap();
    Path::new(get_cargo_registry_path().as_str()).read_dir().ok().into_iter()
        .flatten()
        .filter_map(|s|{
            s.ok()
        })
        .map(|s|{
            s.path()
        })
        .filter(|s|{
            s.is_dir()
        })
        .filter(|s|{
            index_str.is_match(&s.file_name().unwrap_or_default().to_string_lossy().to_string())
        })
        .filter_map(|s|{
            s.read_dir().ok()
        })
        .flatten()
        .filter_map(|s|{
            s.ok()
        })
        .map(|s|{
            s.path()
        })
        .filter(|s|{
            s.is_dir()
        })
        .filter_map(|s|{
            let s1 = s.file_name().unwrap_or_default().to_string_lossy().to_string();
            if let Some(c) = crate_str.captures(&s1){
                Some((s, c.get(1)?.as_str().to_owned()))
            }else{
                None
            }
        })
        .inspect(|s|{
            println!("{}",s.0.to_string_lossy().to_string());
        })
        .collect()
}
fn build_mod_space(m : &Vec<&ModNode>)->Vec<String>{
    m.iter()
        .map(|d|{
            d.name.to_owned()
        })
        .collect()
}
fn build_mod_feature(m : &Vec<&ModNode>)->Vec<String>{
    m.iter()
        .map(|d|{
            d.features.to_owned()
        })
        .flatten()
        .collect()
}
fn walk_root(root : &ModNode, parent : &Vec<&ModNode>, proc : &mut impl FnMut(&Func,&Vec<&ModNode>), recurse : bool){
    //println!("into mod {}, feature: {}, function: {}, children: {}",root.name,root.features.join(","), root.features.len(),root.children.len());
    for f in &root.function{
        proc(f, parent);
    }
    if recurse{
    for m in &root.children{
        //println!("into mod {}", m.name);
        let mut tmp = parent.clone();
        tmp.push(m);
        walk_root(&m, &tmp, proc, recurse);
    }
    }
}
mod tests{
#[test]
fn env_path() {
    let p = super::get_cargo_registry_path();
    println!("{}",p);
    assert!(!p.is_empty());
}
#[test]
fn walk_source(){
    let ds = super::get_crate_dirs("windows");
    assert!(!ds.is_empty());
    let p1 = &ds.last().unwrap();
    let root = super::get_mod_from_source(&p1.0.join("src/Windows"));
    let mut proc = |f : &super::Func,parent : &Vec<&super::ModNode>|{
        println!("fn: {}({}), mod: {}, feature: {}", f.name,f.sig, super::build_mod_space(&parent).join("::"),[super::build_mod_feature(&parent),f.feature.clone()].concat().join(","));
    };
    if let Some(r) = root{
        println!("start walk mod=========================>");
        let init = vec![];
        super::walk_root(&r,&init, &mut proc, true);
    }

}
}
