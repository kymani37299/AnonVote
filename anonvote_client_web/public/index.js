import { generate_key_pair  } from "./anonvote_wasm.js";

window.addEventListener('load', setup);

function setup() {
    const validateIdSection = document.getElementById("validateIdSection");
    validateIdSection.onclick = () => showSection('validate');

    const registerSection = document.getElementById("registerSection");
    registerSection.onclick = () => showSection('register');

    const voteSection = document.getElementById("voteSection");
    voteSection.onclick = () => showSection('vote');

    const validateIdButton = document.getElementById("validateIdButton");
    validateIdButton.onclick = () => validateID();

    const registerUserButton = document.getElementById("registerUserButton");
    registerUserButton.onclick = () => registerUser();

    const submitVoteButton = document.getElementById("submitVoteButton");
    submitVoteButton.onclick = () => submitVote();
}

function showSection(section) {
    // Hide all sections
    const sections = document.querySelectorAll('.section');
    sections.forEach(sec => sec.style.display = 'none');
    
    // Show selected section
    document.getElementById(section).style.display = 'block';
}

function downloadFile(content) {
    const secretKey = content;
    const blob = new Blob([secretKey], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    
    const link = document.createElement('a');
    link.href = url;
    link.download = 'secret_key.anonvote';
    link.click();
}

function api_call(path, body_str, onOk, onError) {
    fetch(path, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: body_str
      })
    .then(response => {
        if (!response.ok) {
            return response.json().then(errorData => {
                const errorMessage = errorData.details || 'An unknown error occurred.';
                throw new Error(`${errorMessage}`);
            });
        }
        return response.json();
    })
    .then(data => onOk(data))
    .catch(error => onError(error));
}

function validateID() {
    const idFile = document.getElementById('idFile').files[0];
    const idNumber = document.getElementById('idNumber').value;
    const message = document.getElementById('validateMessage');
    
    if (!idFile || !idNumber) {
        message.innerHTML = 'Please upload an ID image and enter the ID number.';
        message.style.color = 'red';
    } else {
        api_call(
            '/validate_id', 
            JSON.stringify({ id: idNumber }),
            data => {
                const registrationKey = data.registrationKey;
                message.innerHTML = 'ID validated successfully! Registration key: ' + registrationKey;
                message.style.color = 'green';
            },
            error => {
                message.innerHTML = error;
                message.style.color = 'red';
            }
        );
    }
}

function registerUser() {
    const registrationCode = document.getElementById('registrationCode').value;
    const message = document.getElementById('registerMessage');

    if(!registrationCode || registrationCode.trim() == '') {
        message.innerHTML = 'Invalid registration code.';
        message.style.color = 'red';
    } else {
        const generated_key = generate_key_pair();  
        console.log("Keys generated!");
        console.log("WASM: ", generated_key);
        console.log("Secret: ", generated_key.secret_key.secret());
        console.log("a: ", generated_key.public_key.a());
        console.log("b: ", generated_key.public_key.b());
        console.log("alpha: ", generated_key.public_key.alpha());
        console.log("beta: ", generated_key.public_key.beta());
        const registerReq = {
            registrationKey : registrationCode,
            a : generated_key.public_key.a(),
            b : generated_key.public_key.b(),
            alpha : generated_key.public_key.alpha(),
            beta : generated_key.public_key.beta()
        };

        console.log("Sending request: " , JSON.stringify(registerReq));

        api_call(
            '/register', 
            JSON.stringify(registerReq),
            _ => {
                downloadFile(JSON.stringify(generated_key));
                message.innerHTML = 'Registered successfully! Please download private key file.';
                message.style.color = 'green';
            },
            error => {
                message.innerHTML = error;
                message.style.color = 'red';
            }
        );
    }

    // const validCode = 'SECRET123'; // Example valid code
    
    // if (registrationCode === validCode) {
    //     const secretKey = 'SECRET_KEY_' + Math.random().toString(36).substr(2, 9);
    //     const blob = new Blob([secretKey], { type: 'text/plain' });
    //     const url = URL.createObjectURL(blob);
        
    //     const link = document.createElement('a');
    //     link.href = url;
    //     link.download = 'secret_key.txt';
    //     link.click();
        
    //     message.innerHTML = 'Registration successful! Downloading secret key.';
    //     message.style.color = 'green';
    // } else {
    //     message.innerHTML = 'Invalid registration code.';
    //     message.style.color = 'red';
    // }
}

function submitVote() {
    const secretKeyFile = document.getElementById('secretKeyFile').files[0];
    const voteOption = document.querySelector('input[name="vote"]:checked');
    const message = document.getElementById('voteMessage');
    
    if (!secretKeyFile) {
        message.innerHTML = 'Please upload your secret key file.';
        message.style.color = 'red';
    } else if (!voteOption) {
        message.innerHTML = 'Please select a vote option.';
        message.style.color = 'red';
    } else {
        message.innerHTML = 'Vote submitted successfully!';
        message.style.color = 'green';
    }
}
