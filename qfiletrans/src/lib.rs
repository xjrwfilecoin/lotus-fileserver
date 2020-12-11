// mod mini_client;
// mod mini_server;
// mod protocol;
// mod file_manager;
//
// use mini_server::start_server;

//mod file_manager;

// mod file_manager;
// mod protocol;


use std::net::TcpStream;
use std::net::TcpListener;
use async_std::prelude::*;
use anyhow::Result;
use async_std::io;
use thread_priority::*;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use async_std::task;

use std::thread;
use std::ops::{Deref, DerefMut, Add};
use anyhow::{*};
use structopt::StructOpt;
use rand::prelude::*;
use std::io::{Read, Write, Cursor};
use log::{info, error};
use std::fs::{OpenOptions};
use std::path::PathBuf;
use std::process::Command;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::str::FromStr;

/// Search for a pattern in a file and display the lin that contain it.
#[derive(StructOpt)]
struct Cli {
    /// The pattern to look for
    pub mode: String,

    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    pub path: std::path::PathBuf,

    /// The path to the file to read
    #[structopt(short = "d", long = "dest")]
    pub dest: String,
}

const buf_len: usize = 200 as usize * 1024 * 1024;



#[derive(Clone, Eq, PartialEq, Debug)]
pub struct FileInfo {
    file_len: u64,
    file_name: String,
}

impl FileInfo {
    pub fn new(file_len: u64, file_name: String) -> Self {
        FileInfo {
            file_len,
            file_name: file_name.clone(),
        }
    }
    pub fn to_vec(&self) -> Vec<u8> {
        let mut out_buf = vec![0u8; 1024];
        let mut rdr = Cursor::new(&mut out_buf[..]);
        rdr.write_u64::<BigEndian>((&self).file_len);
        rdr.write_u32::<BigEndian>((&self).file_name.len() as u32);
        (&mut out_buf[12..12 + self.file_name.len()]).clone_from_slice(&self.file_name.clone().into_bytes()[..]);
        out_buf
    }
}

impl From<&[u8]> for FileInfo {
    fn from(buf: &[u8]) -> Self {
        let mut rdr = Cursor::new(&buf[..]);
        let file_len = rdr.read_u64::<BigEndian>().unwrap();
        let name_len: u32 = rdr.read_u32::<BigEndian>().unwrap();
        let file_name = String::from_utf8_lossy(&buf[12..12 + name_len as usize]);
        FileInfo {
            file_len,
            file_name: file_name.to_string(),
        }
    }
}

impl From<Vec<u8>> for FileInfo {
    fn from(buf: Vec<u8>) -> Self {
        let mut rdr = Cursor::new(&buf[..]);
        let file_len = rdr.read_u64::<BigEndian>().unwrap();
        let name_len: u32 = rdr.read_u32::<BigEndian>().unwrap();
        let file_name = String::from_utf8_lossy(&buf[12..12 + name_len as usize]);
        FileInfo {
            file_len,
            file_name: file_name.to_string(),
        }
    }
}

impl From<&Vec<u8>> for FileInfo {
    fn from(buf: &Vec<u8>) -> Self {
        let mut rdr = Cursor::new(&buf[..]);
        let file_len = rdr.read_u64::<BigEndian>().unwrap();
        let name_len: u32 = rdr.read_u32::<BigEndian>().unwrap();
        let file_name = String::from_utf8_lossy(&buf[12..12 + name_len as usize]);
        FileInfo {
            file_len,
            file_name: file_name.to_string(),
        }
    }
}


pub fn start_server(host:&str,parent_path: std::path::PathBuf) {
    let listener = TcpListener::bind(host).unwrap();

    let state = Arc::new(RwLock::new(0u64));
    let state_read = state.clone();
    thread::spawn(move||{
        let mut offset = 0u64;
        let mut step = 0u8;
        loop {
            thread::sleep(Duration::from_millis(200));
            {
                let st = state_read.read().unwrap();
                if step % 5 == 4 {
                    let mut s = (*st - offset) as f64 / 1024.0f64;
                    if s == 0f64 {
                        continue;
                    }
                    let mut u = "KiB";
                    if s > 1024f64 {
                        u = "MiB";
                        s = s / 1024.0;
                    }
                    if s > 1024f64 {
                        u = "GiB";
                        s = s / 1024.0;
                    }

                    log::trace!("speed: {}{}/s",s.ceil(),u);
                    offset = *st;
                    step = 0;
                }
            }
            step += 1;
        }
    });

    let limit_sleep_ms = get_limit_sleep();
    log::info!("listen on:{},root:{}",host,parent_path.clone().to_str().unwrap());
    for stream in listener.incoming() {
        let parent_path = parent_path.clone();
        let state = state.clone();
        thread::spawn(move || {

            let set_res = set_current_thread_priority(ThreadPriority::Min);
            if !set_res.is_ok()  {
                info!("set store thread priority failed: {:?}", set_res.unwrap_err())
            }

            let stream = stream.unwrap();
            let (reader, writer) = &mut (&stream, &stream);
            let mut read_head = || -> Result<FileInfo>{
                let cur_read_offset = 0;
                let mut buffer =vec![0u8;1024];
                while cur_read_offset < 1024 {
                    match reader.read(&mut buffer[cur_read_offset..1024]) {
                        Ok(read_size) => {
                            if read_size == 0 {
                                return Err(anyhow!("socket read end"));
                            } else {
                                if cur_read_offset + read_size >= 1024 {
                                    return Ok(FileInfo::from(&buffer[..1024]));
                                } else {
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            return Err(anyhow!("error read failed :{}", e.to_string()));
                        }
                    }
                }
                return Err(anyhow!("should never be here"));
            };
            match read_head() {
                Ok(file_info) => {

                    let file_name =  parent_path.join(file_info.file_name);
                    let handler= reader.read_u8().unwrap();
                    if handler == 1 {
                        if let Err(e) = std::fs::remove_dir_all(file_name.parent().unwrap()){
                            log::warn!("remove_dir all,[{}]-->failed:{:?}",file_name.parent().unwrap().to_str().unwrap(),e);
                        }
                        else{
                            let file_txt = format!("{}.txt",file_name.parent().unwrap().to_str().unwrap());
                            log::warn!("remove file:{}",file_txt);
                            if let Err(e) = std::fs::remove_file(file_txt.clone()) {
                                log::error!("remove file [{}] failed",file_txt);
                            };
                            log::warn!("dir:[{}] removed",file_name.parent().unwrap().to_str().unwrap());
                            if let Ok(path) = std::env::var("WORKER_PATH") {
                                let sealed_path = PathBuf::from(path).join("sealed");
                                let name = file_name.parent().unwrap().file_name().unwrap().to_str().unwrap();
                                if let Err(e) = std::fs::remove_file(sealed_path.join(name)){
                                    log::error!("remove file [{}/{}] failed",sealed_path.to_str().unwrap(),name);
                                };
                            };
                        };
                        return;
                    }
                    if !std::path::Path::exists(file_name.parent().unwrap()) {
                        Command::new("mkdir")
                            .arg("-p")
                            .arg(&file_name.parent().unwrap().to_str().unwrap())
                            .output()
                            .expect(format!("failed to create cache path,{}",&file_name.parent().unwrap().to_str().unwrap()).as_str());
                    }
                    let mut open_option = OpenOptions::new();
                    if let Ok(mut file) = open_option.write(true)
                        .create(true)
                        .open(&file_name) {
                        let mut buffer = vec![0u8; buf_len];
                        let mut cur_read_offset = 0usize;

                        // let ring = rio::new().expect("");

                        let file_len = file_info.file_len;
                        file.set_len(file_len);

                        let mut file_len = 0u64;
                        loop {
                            match reader.read(&mut buffer[cur_read_offset..buf_len]) {
                                Ok(read_size) => {

                                    file_len += read_size as u64;
                                    if read_size == 0 {
                                        info!("read finished!");
                                        match file.write(&mut buffer[0..cur_read_offset]) {
                                            Ok(write_size) => {
                                                if write_size != cur_read_offset {
                                                    error!("error in write file size unmatched:{}/{}", read_size, cur_read_offset);
                                                }
                                                let file_txt = format!("{}.txt",&file_name.parent().unwrap().to_str().unwrap());
                                                info!("uploaded file:{}",file_name.to_path_buf().to_str().unwrap());
                                                //file.sync_data();
                                                if file_len == file_info.file_len {
                                                    let mut file = OpenOptions::new()
                                                        .create(true)
                                                        .write(true)
                                                        .append(true)
                                                        .open(PathBuf::from(file_txt))
                                                        .unwrap();

                                                    if let Err(e) = writeln!(file, "{:?}",&file_name.display()) {
                                                        eprintln!("Couldn't write to file: {}", e);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!("error in write file :{}", e.to_string());
                                            }
                                        }

                                        break;
                                    }
                                    let mut st = state.write().unwrap();
                                    *st += read_size as u64;
                                    if cur_read_offset + read_size < buf_len {
                                        cur_read_offset += read_size;
                                    } else {
                                        cur_read_offset += read_size;
                                        match file.write(&mut buffer[..]) {
                                            Ok(write_size) => {
                                                if write_size != cur_read_offset {
                                                    error!("error in write file size unmatched:{}/{}", read_size, cur_read_offset);
                                                    break;
                                                }

                                                thread::sleep(std::time::Duration::from_millis(limit_sleep_ms as u64));
                                                //file.sync_data();
                                            }
                                            Err(e) => {
                                                error!("error in write file :{}", e.to_string());
                                            }
                                        }
                                        cur_read_offset = 0;
                                    }
                                }
                                Err(e) => {
                                    error!("error in read:{}", e.to_string())
                                }
                            }
                        }


                    } else {
                        error!("error in write:{:?}", &file_name)
                    }

                }
                Err(e) => {
                    error!("error in write file :{}", e.to_string());
                }
            }

        });
    }
}

fn get_limit_sleep() -> u64 {
    let limit_sleep_ms = match std::env::var("FIL_TRANS_SLEEP")
    {
        Ok(ms) => {
            match ms.parse::<u64>() {
                Ok(v) => v,
                Err(_e) => {
                    5u64
                }
            }
        },
        Err(_e) => {
            5u64
        }
    };
    limit_sleep_ms
}

///remove remote file
pub fn start_remove(dest: String,cut_file_name: &std::path::PathBuf) {
    // let remove = vec![0u8;32];
    match TcpStream::connect(&dest) {
        Ok(stream)=>{
            let (_reader, writer) = &mut (&stream, &stream);
            let file_info = FileInfo::new(32,String::from(cut_file_name.to_str().unwrap() ));
            match writer.write_all(&(&file_info).to_vec()[..]) {
                Ok(_)=>{
                    if let Err(e) = writer.write_u8(1u8){//协议行为标识--删除
                        error!("error in write stream:{}", e.to_string());
                    };
                },
                Err(e)=>{
                    error!("error in write stream:{}", e.to_string());
                }
            }
        },
        Err(e) =>{
            log::warn!("error in connecting to {},error:{}",&dest,e.to_string());
            if let Ok(_) = start_remove_retry(dest,cut_file_name,1){
                log::info!("start_remove_retry OK");
            };
        }
    }
}

fn start_remove_retry(dest: String,cut_file_name: &std::path::PathBuf,retry:u32) -> Result<()>{
    match TcpStream::connect(&dest) {
        Ok(stream)=>{
            let (_reader, writer) = &mut (&stream, &stream);
            let file_info = FileInfo::new(32,String::from(cut_file_name.to_str().unwrap() ));
            match writer.write_all(&(&file_info).to_vec()[..]) {
                Ok(_)=>{
                    if let Err(e) = writer.write_u8(1u8){//协议行为标识--删除
                        error!("error in write stream:{}", e.to_string());
                    };
                    Ok(())
                },
                Err(e)=>{
                    error!("error in write stream:{}", e.to_string());
                    Ok(())
                }
            }
        },
        Err(e) =>{
            if retry < 10 {
                let dest = dest.clone();
                let cut_file_name = cut_file_name.clone();
                thread::spawn(move||{
                    thread::sleep(Duration::from_secs(30));
                    if let Err(e) = start_remove_retry(dest.clone(),&cut_file_name,retry + 1){

                    };
                });
            }
            else{
                log::error!("error in connecting to {},error:{}",&dest,e.to_string());
            }
            Err(anyhow!("error in connecting to {},error:{}",&dest,e.to_string()))
        }
    }
}
// pub fn start_upload_async(dest: String, real_file: &std::path::PathBuf, cut_file_name: &std::path::PathBuf,sector_id:&str){
//     thread::spawn(move||{
//         let f_his = PathBuf::from(format!("./{}.his", sector_id));
//         let n = std::time::Instant::now();
//
//         start_upload(dest,real_file,&cut_file_name.clone());
//         if let Ok(mut his) = std::fs::OpenOptions::new().append(true).create(true).open(f_his) {
//             //记录文件传输耗时日志
//             let mut ss = n.elapsed().as_secs();
//             let mut mm = 0;
//             if ss > 60 {
//                 mm = ss / 60;
//                 ss = ss % 60;
//             }
//             let line = format!("[{}:{}] scp {} {}:{}\r\n", mm, ss, real_file.to_str().unwrap(), cut_file_name.clone().to_str().unwrap(), dest);
//             his.write(line.as_bytes()).unwrap();
//         }
//     });
// }

pub fn start_upload_inner(dest: String, real_file: &std::path::PathBuf, cut_file_name: &std::path::PathBuf,retry:u32){
    let mut buffer = vec![0u8; 64 * 1024 * 1024];
    // let limit_sleep_ms = get_limit_sleep();
    info!("connecting to {}",&dest);

    let retry = retry + 1;
    let do_retry = || {
        if retry < 10 {
            let dt = dest.clone();
            let r_f = real_file.clone();
            let c_f = cut_file_name.clone();
            log::warn!("retry-->{}",retry);
            thread::sleep(Duration::from_secs(1<<retry));
            if false == r_f.exists() {
                if let Ok(worker_path) = std::env::var("WORKER_PATH") {
                    let mut root = PathBuf::from_str(&worker_path[..]).unwrap();
                    if r_f.to_str().unwrap_or("").contains("/cache/"){
                        root = root.join("cache");
                    }
                    let file_name = r_f.file_name().unwrap();
                    let dir = r_f.parent().unwrap().file_name().unwrap();
                    let real_file = root.join(dir).join(file_name);
                    if real_file.exists() {
                        start_upload_inner(dt,&real_file,&c_f,retry);
                    }
                    else{
                        log::error!("retry file not exists:{:?} or {:?}",r_f,real_file);
                    }
                }
            }
            else{
                start_upload_inner(dt,&r_f,&c_f,retry);
            }
        }
    };
    if let Ok(stream) = TcpStream::connect(&dest) {
        let (_reader, writer) = &mut (&stream, &stream);
        let mut open_option = OpenOptions::new();

        match open_option.read(true).open(&real_file) {
            Ok(mut file) => {

                let file_info = FileInfo::new(file.metadata().unwrap().len(),String::from(cut_file_name.to_str().unwrap() ));
                match writer.write_all(&(&file_info).to_vec()[..]) {
                    Ok(_) =>{
                        //协议行为标识--上传
                        if let Err(e) = writer.write_u8(0u8) {
                            do_retry();
                            return;
                        }
                        loop {
                            match file.read(&mut buffer[..]) {
                                Ok(read_size) => {
                                    if read_size == 0 {
                                        info!("readfinished");
                                        break;
                                    } else {
                                        match writer.write_all(&buffer[0..read_size]) {
                                            Ok(_) => {
                                                // thread::sleep(Duration::from_millis(limit_sleep_ms));
                                            }
                                            Err(e) => {
                                                error!("error in write stream:{}", e.to_string());
                                                do_retry();
                                                break;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("error in read file:{}", e.to_string());
                                    do_retry();
                                    break;
                                }
                            }
                        }
                    },
                    Err(e) => {
                        error!("error in write FileHeader:{}", e.to_string());
                        do_retry();
                    }
                }

            }
            Err(e) => {
                error!("read file {:?} failed:{:?}", &real_file.display(), e.to_string());
                do_retry();
            }
        }
    }else{
        error!("error in connecting to {}",&dest);
        do_retry();
    };
}
pub fn start_upload(dest: String, real_file: &std::path::PathBuf, cut_file_name: &std::path::PathBuf) {
    start_upload_inner(dest,real_file,cut_file_name,0);
}

// fn main() {
//     let args = Cli::from_args();
//     if args.mode == String::from("server") {
//         start_server(args.path);
//     } else if args.mode == String::from("client") {
//         start_client(args.dest, args.path.parent().unwrap().to_path_buf(), PathBuf::from(args.path.file_name().unwrap()));
//     }
// }

#[cfg(test)]
mod Test {
    use crate::{start_server,start_upload,FileInfo,start_remove};
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_file_info() {
        env_logger::init();
        let file_info = FileInfo::new(2 * 1024 * 1024, String::from("thies is a test/with path/filename is.txt"));
        let result = file_info.to_vec();
        let worker_path = std::env::var("WORKER_PATH").unwrap_or("/tmp".to_owned());
        //
        // let t1 = thread::spawn(||{
        //     start_server("0.0.0.0:28081",PathBuf::from(worker_path));
        // });
        let t2 = thread::spawn(||{
            start_upload(String::from("localhost:8081"),&PathBuf::from("/opt/ssd_pool/filecoin-proof-parameters-v28.tar"),&PathBuf::from(r"cache/s-t01000-1/filecoin-proof-parameters-v28.tar"));
            // start_remove(String::from("localhost:28081"),&PathBuf::from(r"cache/s-t01000-1/remove"));
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        });
        // let t2 = thread::spawn(||{
        //     start_remove(String::from("localhost:28081"),&PathBuf::from(r"s-t01000-1\lotus-master.tar"));
        // });
        t2.join();
        thread::sleep(Duration::from_secs(2));
        // t1.join();
    }
}