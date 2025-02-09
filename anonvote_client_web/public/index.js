import { generate_key_pair, json_to_key_pair, key_pair_to_json  } from "./anonvote_wasm.js";

const keyFileName = "userKey.anonvote";

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
    link.download = keyFileName;
    link.click();
}

function readFileAsText(file) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
  
      reader.onload = function(event) {
        resolve(event.target.result);
      };
  
      reader.onerror = function(event) {
        reject(new Error("Error reading file"));
      };
  
      reader.readAsText(file);
    });
  }

async function readJSONFile(file) {
    if(!file) {
        return undefined;
    }

    try {
        const fileContent = await readFileAsText(file);
        const jsonObject = JSON.parse(JSON.parse(fileContent));
        return jsonObject;
      } catch (error) {
        console.log(error);
        return undefined;
      }
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
        const registerReq = {
            registrationKey : registrationCode,
            a : generated_key.public_key.a(),
            b : generated_key.public_key.b(),
            alpha : generated_key.public_key.alpha(),
            beta : generated_key.public_key.beta()
        };

        api_call(
            '/register', 
            JSON.stringify(registerReq),
            _ => {
                downloadFile(JSON.stringify(key_pair_to_json(generated_key)));
                message.innerHTML = 'Registered successfully! Please download private key file.';
                message.style.color = 'green';
            },
            error => {
                message.innerHTML = error;
                message.style.color = 'red';
            }
        );
    }
}

async function submitVote() {
    const secretKeyFile = document.getElementById('secretKeyFile').files[0];
    const voteOption = document.querySelector('input[name="vote"]:checked');
    const message = document.getElementById('voteMessage');

    const voteOptionInt = parseInt(voteOption.value);

    if (!secretKeyFile) {
        message.innerHTML = 'Please upload your user key file.';
        message.style.color = 'red';
        return;
    }
    
    if (!voteOption || !voteOptionInt) {
        message.innerHTML = 'Please select a vote option.';
        message.style.color = 'red';
        return;
    }

    const jsonKeyPair = await readJSONFile(secretKeyFile);
    const keyPair = json_to_key_pair(jsonKeyPair);
    
    if(!keyPair) {
        message.innerHTML = 'Please upload valid user key file.';
        message.style.color = 'red';
        return;
    }

    const challengeReq = keyPair.public_key.generate_challenge_request();

    let voteReq = {
        vote : voteOptionInt,
        a : keyPair.public_key.a(),
        b : keyPair.public_key.b(),
        alpha : keyPair.public_key.alpha(),
        beta : keyPair.public_key.beta(),
        ka : challengeReq.ka(),
        kb : challengeReq.kb()
    };

    api_call(
        '/vote', 
        JSON.stringify(voteReq), 
        (data) => {
            message.innerHTML = 'Authentication...';
            message.style.color = 'blue';
            validateVote(voteOptionInt, keyPair, challengeReq, data.challenge, data.authSessionId);
        },
        (error) => {
            message.innerHTML = error;
            message.style.color = 'red';
        });
}

function validateVote(vote, keyPair, challengeReq, challenge, session_id) {
    const message = document.getElementById('voteMessage');

    let solution = keyPair.private_key.solve(challengeReq.k(), challenge);

    let validationReq = {
        auth_session_id : session_id,
        vote : vote,
        solution : solution
    };

    api_call(
        '/validate_vote',
        JSON.stringify(validationReq),
        _ => {
            message.innerHTML = 'Voting finished!';
            message.style.color = 'green';
        },
        error => {
            message.innerHTML = error;
            message.style.color = 'red';
        }
    );
}
