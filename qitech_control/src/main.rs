use qitech_lib::ethercat_hal::start_ethercat_thread;

fn main(){
	/*
		Only start thread when interface is found 
	*/
	let res = start_ethercat_thread("eth0");
	let result = res.0;
	let ecat_controller = result.0;
	let ecat_channel = result.1;



}