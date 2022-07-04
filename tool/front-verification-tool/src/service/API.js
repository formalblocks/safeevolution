import axios from 'axios';

export default axios.create({
	baseURL: "http://localhost:8000/",
	withCredentials: false,
	headers: {
		'Content-Type': 'application/json;charset=utf-8',
		'Accept': 'application/json',
		"Access-Control-Allow-Origin": "*",
	}
});



// export default axios.create({
// 	baseURL: "http://localhost:8000/",
// 	withCredentials: false,
// 	headers: {
// 		'Content-Type': 'application/json;charset=utf-8',
// 		'Accept': 'application/json',
// 		"Access-Control-Allow-Origin": "*",
// 		'Access-Control-Allow-Credentials':true
// 	}
// });