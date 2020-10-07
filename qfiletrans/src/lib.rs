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
use std::sync::{Arc, Mutex};
use std::time::Duration;
use async_std::task;

use std::thread;
use std::ops::{Deref, DerefMut};
use anyhow::{*};
use structopt::StructOpt;
use rand::prelude::*;
use std::io::{Read, Write, Cursor};
use log::{info, error};
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::Command;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

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
    //  let mut incoming = listener.incoming();

    let total_threads = Arc::new(Mutex::new(0));
    for stream in listener.incoming() {
        let parent_path = parent_path.clone();
        let total_threads = total_threads.clone();
        thread::spawn(move || {
            {
                let mut cnts = total_threads.lock().unwrap();
                let cnts = cnts.deref_mut();
                *cnts += 1;
            }

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
                                return Err(anyhow!("socket closed"));
                            } else {
                                if cur_read_offset + read_size >= 1024 {
                                    return Ok(FileInfo::from(&buffer[..1024]));
                                } else {
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            return Err(anyhow!("error in write file :{}", e.to_string()));
                        }
                    }
                }
                return Err(anyhow!("should never be here"));
            };
            match read_head() {
                Ok(file_info) => {
                    let file_name =  parent_path.join(file_info.file_name);
                    Command::new("mkdir")
                        .arg("-p")
                        .arg(&file_name.parent().unwrap().to_str().unwrap())
                        .output()
                        .expect("failed to create cache path");
                    let mut open_option = OpenOptions::new();
                    if let Ok(mut file) = open_option.write(true).create(true).open(parent_path.join(&file_name)) {
                        let mut buffer = vec![0u8; buf_len];
                        let mut cur_read_offset = 0usize;

                        file.set_len(file_info.file_len);
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
                                                info!("create file:{}",file_txt);
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
                                                let cnts = {
                                                    let mut cnts = total_threads.lock().unwrap();
                                                    *cnts.deref_mut()
                                                };
                                                thread::sleep(std::time::Duration::from_millis(400 * cnts as u64));
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

            {
                let mut cnts = total_threads.lock().unwrap();
                let cnts = cnts.deref_mut();
                if *cnts > 0 {
                    *cnts -= 1;
                }
            }
        });
    }
}

pub fn start_upload(dest: String, real_file: &std::path::PathBuf, cut_file_name: &std::path::PathBuf) {
    let mut buffer = vec![0u8; 64 * 1024 * 1024];
    println!("connecting to {}",&dest);
    if let Ok(stream) = TcpStream::connect(&dest) {
        let (reader, writer) = &mut (&stream, &stream);
        let mut open_option = OpenOptions::new();

        match open_option.read(true).open(&real_file) {
            Ok(mut file) => {

                let file_info = FileInfo::new(file.metadata().unwrap().len(),String::from(cut_file_name.to_str().unwrap() ));
                match writer.write_all(&(&file_info).to_vec()[..]) {
                    Ok(_) =>{
                        loop {
                            match file.read(&mut buffer[..]) {
                                Ok(read_size) => {
                                    if read_size == 0 {
                                        info!("readfinished");
                                        break;
                                    } else {
                                        match writer.write_all(&buffer[0..read_size]) {
                                            Ok(_) => {
                                                thread::sleep(Duration::from_millis(200));
                                            }
                                            Err(e) => {
                                                error!("error in write stream:{}", e.to_string())
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("error in read file:{}", e.to_string())
                                }
                            }
                        }
                    },
                    Err(e) => {
                        error!("error in write FileHeader:{}", e.to_string());

                    }
                }

            }
            Err(e) => {
                error!("read file {:?} failed:{:?}", &real_file.display(), e.to_string())
            }
        }
    }else{
        println!("error in connecting to {}",&dest);
    };

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
    use crate::{start_server,start_upload,FileInfo};
    use std::path::PathBuf;
    use std::thread;
    #[test]
    fn test_FileInfo() {
        let file_info = FileInfo::new(2 * 1024 * 1024, String::from("thies is a test/with path/filename is.txt"));
        let result = file_info.to_vec();


        let t1 = thread::spawn(||{
            start_server("0.0.0.0:8081",PathBuf::from("/mnt/data/server"));
        });
        let t2 = thread::spawn(||{
            start_upload(String::from("localhost:8081"),&PathBuf::from("/mnt/ssd/bench/cache/s-t01000-1/sc-02-data-tree-c-1.dat"),&PathBuf::from("s-t01000-1/sc-02-data-tree-c-1.dat"));
        });
        t2.join();
        t1.join();
    }
}