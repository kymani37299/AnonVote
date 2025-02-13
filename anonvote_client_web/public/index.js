import { generate_key_pair, json_to_key_pair, key_pair_to_json, convert_to_uint8_array } from "./anonvote_wasm.js";

const keyFileName = "userKey.anonvote";

window.addEventListener('load', setup);

async function setup() {
    const validateIdSection = document.getElementById("validateIdSection");
    validateIdSection.onclick = () => showSection('validate');

    const registerSection = document.getElementById("registerSection");
    registerSection.onclick = () => showSection('register');

    const voteSection = document.getElementById("voteSection");
    voteSection.onclick = () => showSection('vote');

    const resultsSection = document.getElementById("resultsSection");
    resultsSection.onclick = () => showResults();

    const validateIdButton = document.getElementById("validateIdButton");
    validateIdButton.onclick = () => validateID();

    const registerUserButton = document.getElementById("registerUserButton");
    registerUserButton.onclick = () => registerUser();

    const submitVoteButton = document.getElementById("submitVoteButton");
    submitVoteButton.onclick = () => submitVote();

    const voteOptionsDiv = document.getElementById("voteOptions");

    fetch('/vote_options', {
        method: 'GET'
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
    .then(data => {
        let voteOptions = data.options;
        voteOptions.forEach((optionText, index) => {
            const optionValue = index + 1; // Value 1, 2, 3
            
            const optionDiv = document.createElement("div");
    
            const radioInput = document.createElement("input");
            radioInput.type = "radio";
            radioInput.id = `option${optionValue}`;
            radioInput.name = "vote";
            radioInput.value = optionValue;
    
            const label = document.createElement("label");
            label.htmlFor = `option${optionValue}`;
            label.textContent = optionText;
    
            optionDiv.appendChild(radioInput);
            optionDiv.appendChild(label);
            voteOptionsDiv.appendChild(optionDiv);
        });
    })
    .catch(error => {
        const errorLabel = document.createElement("p");
        errorLabel.textContent = error;
        errorLabel.style.color = 'red';
        voteOptionsDiv.appendChild(errorLabel);
    });
}

function showSection(section) {
    // Hide all sections
    const sections = document.querySelectorAll('.section');
    sections.forEach(sec => sec.style.display = 'none');
    
    // Show selected section
    document.getElementById(section).style.display = 'block';
}

async function showResults() {
    showSection('results');

    const resultsDiv = document.getElementById("resultsDiv");
    resultsDiv.innerHTML = "";

    let loadingLabel = document.createElement("p");
    loadingLabel.textContent = "Loading results...";

    resultsDiv.appendChild(loadingLabel);

    let voteOptions = null;

    await fetch('/vote_options', {
        method: 'GET'
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
    .then(data => {
        voteOptions = data.options;
    })
    .catch(error => {
        loadingLabel.textContent = error;
        loadingLabel.style.color = 'red';
    });

    if(!voteOptions) {
        return;
    }

    let voteResults = null;

    await fetch('/vote_results', {
        method: 'GET'
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
    .then(data => {
        voteResults = data.votes;
    })
    .catch(error => {
        const errorLabel = document.createElement("p");
        errorLabel.textContent = error;
        errorLabel.style.color = 'red';
        resultsDiv.appendChild(errorLabel);
    });

    if(!voteResults) {
        return;
    }

    let numOptions = Math.min(voteResults.length, voteOptions.length); // Those 2 just be the same but just in case

    let voteCount = 0;
    for(let i=0;i<numOptions;i++) {
        voteCount += voteResults[i];
    }

    resultsDiv.innerHTML = "";

    // Create a styled container
    let resultsContainer = document.createElement("div");
    resultsContainer.style.padding = "10px";
    resultsContainer.style.border = "1px solid #ccc";
    resultsContainer.style.borderRadius = "8px";
    resultsContainer.style.backgroundColor = "#f9f9f9";
    resultsContainer.style.width = "fit-content";
    resultsContainer.style.marginTop = "10px";
    
    // Total votes label
    let totalVotesLabel = document.createElement("h3");
    totalVotesLabel.textContent = `Total Votes: ${voteCount}`;
    totalVotesLabel.style.marginBottom = "10px";
    resultsContainer.appendChild(totalVotesLabel);
    
    // Vote results
    for (let i = 0; i < numOptions; i++) {
        let votePercent = voteCount === 0 ? 0.0 : (100.0 * voteResults[i] / voteCount).toFixed(2);
        
        let resultItem = document.createElement("div");
        resultItem.style.display = "flex";
        resultItem.style.justifyContent = "space-between";
        resultItem.style.padding = "5px 10px";
        resultItem.style.borderRadius = "5px";
        resultItem.style.backgroundColor = "#e0e0e0";
        resultItem.style.marginBottom = "5px";
        
        let label = document.createElement("span");
        label.textContent = voteOptions[i];
    
        let percentage = document.createElement("span");
        percentage.textContent = `${votePercent}%`;
        percentage.style.fontWeight = "bold";
    
        resultItem.appendChild(label);
        resultItem.appendChild(percentage);
        resultsContainer.appendChild(resultItem);
    }
    
    // Append to resultsDiv
    resultsDiv.appendChild(resultsContainer);
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
        if(!generated_key) {
            message.innerHTML = 'Internal error, please try again later';
            message.style.color = 'red';
            return;
        }

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
            validateVote(voteOptionInt, keyPair, challengeReq, data.challenge.data, data.authSessionId);
        },
        (error) => {
            message.innerHTML = error;
            message.style.color = 'red';
        });
}

function validateVote(vote, keyPair, challengeReq, challenge, session_id) {
    const message = document.getElementById('voteMessage');

    let solution = keyPair.private_key.solve(challengeReq.k(), convert_to_uint8_array(challenge));

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
