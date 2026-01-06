(function() {
	function updateTimes() {
		const now = Date.now();
		const elements = document.querySelectorAll(".live-time");

		elements.forEach(el => {
			const ts = parseInt(el.getAttribute("data-ts"));
			const type = el.getAttribute("data-type");

			if (!ts) return;

			let text = "";

			switch (type) {
				case "clock":
					text = "⏲ " + new Date().toLocaleTimeString(undefined, { hour12: false });
					break;
				case "uptime":
					const diff = now - ts;
					const seconds = Math.floor(diff / 1000);
					const minutes = Math.floor(seconds / 60);
					const hours = Math.floor(minutes / 60);
					const days = Math.floor(hours / 24);

					text = `⏱ Host Uptime: ${days} days, ${hours % 24} hours, ${minutes % 60} minutes`;
					break;
				case "smart":
					const date = new Date(ts);
					const today = new Date();

					const isToday = date.getDate() == today.getDate()
											 && date.getMonth() == today.getMonth()
											 && date.getFullYear() == today.getFullYear();

					const yesterday = new Date(today);
					yesterday.setDate(yesterday.getDate()-1);

					const isYesterday = date.getDate() === yesterday.getDate()
													 && date.getMonth() === yesterday.getMonth()
													 && date.getFullYear() === yesterday.getFullYear();

					const timeStr = date.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });

					if (isToday) {
						text = `Today, ${timeStr}`;
					} else if (isYesterday) {
						text = `Yesterday, ${timeStr}`;
					} else {
						const options = { month: "long", year: "numeric" }
						text = date.toLocaleDateString(undefined, options);
					}
					break;
			}

			if (el.innerText != text) el.innerText = text;
		});
	}

	updateTimes();
	setInterval(updateTimes, 1000);
})();

