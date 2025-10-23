const timeElement = document.getElementById("server_time");

if (timeElement) {
	const serverStartTime = parseInt(timeElement.dataset.serverTimestamp, 10);
	const clientStartTime = Date.now();

	function updateClock() {
		const elapsedTime = Date.now() - clientStartTime;
		const currentServerTime = new Date(serverStartTime + elapsedTime);

		const timeString = currentServerTime.toLocaleTimeString('nl-NL');
		timeElement.innerText = `⏲  ${timeString}`;
	}

	setInterval(updateClock, 1000);
};

