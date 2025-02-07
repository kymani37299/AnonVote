document.getElementById('validateButton').addEventListener('click', function() {
    const id = document.getElementById('idInput').value || '';
    fetch('/validate_id', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ id: id })
    })
    .then(response => response.json())
    .then(data => {
        const registrationKey = data.registrationKey;
        document.getElementById('registrationCode').textContent = registrationKey;
    })
    .catch(error => {
      console.error('Error:', error);
    });
  });