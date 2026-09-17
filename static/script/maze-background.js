(() => {
	const canvas = document.getElementById("maze-background");
	if (!(canvas instanceof HTMLCanvasElement)) return;

	const ctx = canvas.getContext("2d");
	if (!ctx) return;

	const NORTH = 1 << 0;
	const EAST  = 1 << 1;
	const SOUTH = 1 << 2;
	const WEST  = 1 << 3;
	const ALL_WALLS = NORTH | EAST | SOUTH | WEST;
	const directions = [
		{ dx:  1, dy:  0, wall: EAST,  opposite: WEST  },
		{ dx: -1, dy:  0, wall: WEST,  opposite: EAST  },
		{ dx:  0, dy:  1, wall: SOUTH, opposite: NORTH },
		{ dx:  0, dy: -1, wall: NORTH, opposite: SOUTH },
	];

	let config;
	let viewportWidth = 0;
	let viewportHeight = 0;
	let columns = 0;
	let rows = 0;
	let offsetX = 0;
	let offsetY = 0;
	let maze;
	let inMaze;
	let walkDirections;
	let remainingCells = 0;
	let generator;
	let search;
	let finalPath = [];
	let revealedPathLength = 0;
	let phase = "generation";
	let holdUntil = 0;
	let sceneOpacity = 1;
	let lastFrame = 0;
	let resizeTimer;
	let mazeEnabled = true;

	const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

	function cssValue(name, fallback) {
		return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
	}

	function cssNumber(name, fallback) {
		const value = Number.parseFloat(cssValue(name, String(fallback)));
		return Number.isFinite(value) ? value : fallback;
	}

	function readConfig() {
		return {
			wallColor: cssValue("--maze-wall-color", "hsl(0, 0%, 18%)"),
			searchColor: cssValue("--maze-search-color", "hsl(260, 28%, 23%)"),
			pathColor: cssValue("--maze-path-color", "hsl(280, 36%, 30%)"),
			pathCellColor: cssValue("--maze-path-cell-color", "hsl(280, 46%, 45%)"),
			carverTrailColor: cssValue("--maze-carver-trail-color", "hsl(260, 28%, 23%)"),
			carverColor: cssValue("--maze-carver-color", "hsl(260, 42%, 34%)"),
			finderColor: cssValue("--maze-finder-color", "hsl(280, 68%, 56%)"),
			cellSize: Math.max(20, cssNumber("--maze-cell-size", 24)),
			frameMs: Math.max(50, cssNumber("--maze-frame-ms", 62)),
			generationSteps: Math.max(1, cssNumber("--maze-generation-steps", 20)),
			searchSteps: Math.max(1, cssNumber("--maze-search-steps", 4)),
			revealSteps: Math.max(1, cssNumber("--maze-reveal-steps", 3)),
			holdMs: Math.max(1000, cssNumber("--maze-hold-ms", 5000)),
			fadeStep: Math.max(0.01, cssNumber("--maze-fade-step", 0.14)),
		};
	}

	function randomInt(max) {
		return Math.floor(Math.random() * max);
	}

	function indexOf(x, y) {
		return y * columns + x;
	}

	function coordinates(index) {
		return { x: index % columns, y: Math.floor(index / columns) };
	}

	function neighbour(index, directionIndex) {
		const { x, y } = coordinates(index);
		const direction = directions[directionIndex];
		const nx = x + direction.dx;
		const ny = y + direction.dy;
		if (nx < 0 || nx >= columns || ny < 0 || ny >= rows) return -1;
		return indexOf(nx, ny);
	}

	function validDirections(index) {
		const result = [];
		for (let i = 0; i < directions.length; i += 1) {
			if (neighbour(index, i) !== -1) result.push(i);
		}
		return result;
	}

	function chooseUnvisitedCell() {
		if (remainingCells === 0) return -1;
		const start = randomInt(maze.length);
		for (let offset = 0; offset < maze.length; offset += 1) {
			const index = (start + offset) % maze.length;
			if (!inMaze[index]) return index;
		}
		return -1;
	}

	function resetGeneration() {
		const cellCount = columns * rows;
		maze = new Uint8Array(cellCount);
		maze.fill(ALL_WALLS);
		inMaze = new Uint8Array(cellCount);
		walkDirections = new Int8Array(cellCount);
		walkDirections.fill(-1);

		const seed = randomInt(cellCount);
		inMaze[seed] = 1;
		remainingCells = cellCount - 1;
		generator = {
			mode: "choose",
			start: -1,
			current: -1,
			target: -1,
			carver: -1,
		};
		search = null;
		finalPath = [];
		revealedPathLength = 0;
		phase = "generation";
	}

	function generationStep() {
		if (generator.mode === "choose") {
			const start = chooseUnvisitedCell();
			if (start === -1) {
				startSearch();
				return;
			}
			generator.start = start;
			generator.current = start;
			generator.mode = "walk";
			return;
		}

		if (generator.mode === "walk") {
			const choices = validDirections(generator.current);
			const directionIndex = choices[randomInt(choices.length)];
			walkDirections[generator.current] = directionIndex;
			generator.current = neighbour(generator.current, directionIndex);

			if (inMaze[generator.current]) {
				generator.target = generator.current;
				generator.carver = generator.start;
				generator.mode = "carve";
			}
			return;
		}

		const directionIndex = walkDirections[generator.carver];
		const next = neighbour(generator.carver, directionIndex);
		const direction = directions[directionIndex];
		maze[generator.carver] &= ~direction.wall;
		maze[next] &= ~direction.opposite;

		if (!inMaze[generator.carver]) {
			inMaze[generator.carver] = 1;
			remainingCells -= 1;
		}

		generator.carver = next;
		if (next === generator.target) generator.mode = "choose";
	}

	function startSearch() {
		const cellCount = maze.length;
		const scores = new Float64Array(cellCount);
		scores.fill(Number.POSITIVE_INFINITY);
		scores[0] = 0;

		const cameFrom = new Int32Array(cellCount);
		cameFrom.fill(-1);

		search = {
			open: [0],
			openSet: new Set([0]),
			closed: new Uint8Array(cellCount),
			cameFrom,
			scores,
			end: cellCount - 1,
			current: 0,
		};
		phase = "search";
	}

	function heuristic(index) {
		const { x, y } = coordinates(index);
		return columns - 1 - x + (rows - 1 - y);
	}

	function accessibleNeighbours(index) {
		const result = [];
		for (let i = 0; i < directions.length; i += 1) {
			if ((maze[index] & directions[i].wall) !== 0) continue;
			const next = neighbour(index, i);
			if (next !== -1) result.push(next);
		}
		return result;
	}

	function reconstructPath(end) {
		const result = [];
		for (let current = end; current !== -1; current = search.cameFrom[current]) {
			result.push(current);
		}
		return result.reverse();
	}

	function searchStep() {
		if (search.open.length === 0) {
			phase = "hold";
			holdUntil = performance.now() + config.holdMs;
			return;
		}

		let bestPosition = 0;
		let bestScore = Number.POSITIVE_INFINITY;
		for (let i = 0; i < search.open.length; i += 1) {
			const candidate = search.open[i];
			const score = search.scores[candidate] + heuristic(candidate);
			if (score < bestScore) {
				bestScore = score;
				bestPosition = i;
			}
		}

		const current = search.open.splice(bestPosition, 1)[0];
		search.current = current;
		search.openSet.delete(current);
		if (current === search.end) {
			finalPath = reconstructPath(current);
			phase = "reveal";
			return;
		}

		search.closed[current] = 1;
		for (const next of accessibleNeighbours(current)) {
			if (search.closed[next]) continue;
			const tentativeScore = search.scores[current] + 1;
			if (tentativeScore >= search.scores[next]) continue;

			search.cameFrom[next] = current;
			search.scores[next] = tentativeScore;
			if (!search.openSet.has(next)) {
				search.open.push(next);
				search.openSet.add(next);
			}
		}
	}

	function createStaticMaze() {
		maze.fill(ALL_WALLS);
		const visited = new Uint8Array(maze.length);
		const stack = [randomInt(maze.length)];
		visited[stack[0]] = 1;

		while (stack.length > 0) {
			const current = stack[stack.length - 1];
			const choices = validDirections(current).filter((directionIndex) => {
				const next = neighbour(current, directionIndex);
				return !visited[next];
			});

			if (choices.length === 0) {
				stack.pop();
				continue;
			}

			const directionIndex = choices[randomInt(choices.length)];
			const next = neighbour(current, directionIndex);
			const direction = directions[directionIndex];
			maze[current] &= ~direction.wall;
			maze[next] &= ~direction.opposite;
			visited[next] = 1;
			stack.push(next);
		}
	}

	function resize() {
		config = readConfig();
		viewportWidth = window.innerWidth;
		viewportHeight = window.innerHeight;
		const pixelRatio = Math.min(window.devicePixelRatio || 1, 1.5);

		canvas.width = Math.round(viewportWidth * pixelRatio);
		canvas.height = Math.round(viewportHeight * pixelRatio);
		ctx.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);

		columns = Math.max(2, Math.floor(viewportWidth / config.cellSize));
		rows = Math.max(2, Math.floor(viewportHeight / config.cellSize));
		offsetX = (viewportWidth - columns * config.cellSize) / 2;
		offsetY = (viewportHeight - rows * config.cellSize) / 2;
		resetGeneration();

		if (reducedMotion.matches) {
			createStaticMaze();
			phase = "static";
		}
		draw();
	}

	function drawSearch() {
		if (!search) return;
		for (let index = 0; index < search.closed.length; index += 1) {
			if (!search.closed[index]) continue;
			fillCell(index, config.searchColor, 1 / 7);
		}
	}

	function traceGenerationTrail() {
		if (generator.start < 0) return [];
		const end = generator.mode === "carve" ? generator.carver : generator.current;
		if (end < 0) return [];

		const trail = [generator.start];
		let current = generator.start;
		for (let guard = 0; current !== end && guard < maze.length; guard += 1) {
			const directionIndex = walkDirections[current];
			if (directionIndex < 0) break;
			current = neighbour(current, directionIndex);
			if (current < 0) break;
			trail.push(current);
		}
		return trail;
	}

	function fillCell(index, color, sizeRatio = 0.32) {
		const { x, y } = coordinates(index);
		ctx.fillStyle = color;
		const size = Math.max(2, Math.floor(config.cellSize * sizeRatio));
		const inset = (config.cellSize - size) / 2;
		ctx.fillRect(
			offsetX + x * config.cellSize + inset,
			offsetY + y * config.cellSize + inset,
			size,
			size,
		);
	}

	function drawGenerationTrail() {
		if (phase !== "generation") return;
		for (const index of traceGenerationTrail()) {
			fillCell(index, config.carverTrailColor, 1 / 7);
		}
	}

	function drawPath() {
		const length = Math.min(revealedPathLength, finalPath.length);
		if (length === 0) return;
		for (let i = 0; i < length; i += 1) {
			fillCell(finalPath[i], config.pathCellColor, 0.32);
		}

		ctx.beginPath();
		for (let i = 0; i < length; i += 1) {
			const { x, y } = coordinates(finalPath[i]);
			const px = offsetX + (x + 0.5) * config.cellSize;
			const py = offsetY + (y + 0.5) * config.cellSize;
			if (i === 0) ctx.moveTo(px, py);
			else ctx.lineTo(px, py);
		}
		ctx.strokeStyle = config.pathColor;
		ctx.lineWidth = Math.max(1, config.cellSize / 8);
		ctx.lineCap = "round";
		ctx.lineJoin = "round";
		ctx.stroke();
	}

	function drawHead(index, color, sizeRatio = 0.25) {
		if (index < 0) return;
		fillCell(index, color, sizeRatio);
	}

	function drawActiveHeads() {
		if (phase === "generation") {
			const current = generator.mode === "carve" ? generator.carver : generator.current;
			drawHead(current, config.carverColor, 0.25);
		} else if (phase === "reveal" && revealedPathLength > 0) {
			const current = finalPath[Math.min(revealedPathLength, finalPath.length) - 1];
			drawHead(current, config.finderColor);
		}
	}

	function drawWalls() {
		ctx.beginPath();
		for (let y = 0; y < rows; y += 1) {
			for (let x = 0; x < columns; x += 1) {
				const cell = maze[indexOf(x, y)];
				const left = offsetX + x * config.cellSize;
				const top = offsetY + y * config.cellSize;
				const right = left + config.cellSize;
				const bottom = top + config.cellSize;

				if (cell & NORTH) {
					ctx.moveTo(left, top);
					ctx.lineTo(right, top);
				}
				if (cell & WEST) {
					ctx.moveTo(left, top);
					ctx.lineTo(left, bottom);
				}
				if (x === columns - 1 && cell & EAST) {
					ctx.moveTo(right, top);
					ctx.lineTo(right, bottom);
				}
				if (y === rows - 1 && cell & SOUTH) {
					ctx.moveTo(left, bottom);
					ctx.lineTo(right, bottom);
				}
			}
		}
		ctx.strokeStyle = config.wallColor;
		ctx.lineWidth = 1;
		ctx.stroke();
	}

	function draw() {
		ctx.clearRect(0, 0, viewportWidth, viewportHeight);
		ctx.save();
		ctx.globalAlpha = sceneOpacity;
		drawSearch();
		drawGenerationTrail();
		drawPath();
		drawWalls();
		drawActiveHeads();
		ctx.restore();
	}

	function update(now) {
		if (phase === "generation") {
			for (let i = 0; i < config.generationSteps && phase === "generation"; i += 1) {
				generationStep();
			}
		} else if (phase === "search") {
			for (let i = 0; i < config.searchSteps && phase === "search"; i += 1) {
				searchStep();
			}
		} else if (phase === "reveal") {
			revealedPathLength += config.revealSteps;
			if (revealedPathLength > finalPath.length) {
				phase = "hold";
				holdUntil = now + config.holdMs;
			}
		} else if (phase === "hold" && now >= holdUntil) {
			phase = "fade-out";
		} else if (phase === "fade-out") {
			sceneOpacity = Math.max(0, sceneOpacity - config.fadeStep);
			if (sceneOpacity === 0) {
				resetGeneration();
				phase = "fade-in";
			}
		} else if (phase === "fade-in") {
			sceneOpacity = Math.min(1, sceneOpacity + config.fadeStep);
			if (sceneOpacity === 1) phase = "generation";
		}
	}

	function animationFrame(now) {
		if (mazeEnabled && !document.hidden && !reducedMotion.matches && now - lastFrame >= config.frameMs) {
			lastFrame = now;
			update(now);
			draw();
		}
		window.requestAnimationFrame(animationFrame);
	}

	window.addEventListener("resize", () => {
		window.clearTimeout(resizeTimer);
		resizeTimer = window.setTimeout(resize, 200);
	});

	reducedMotion.addEventListener("change", resize);

	const mazeToggle = document.getElementById("maze-toggle");
	if (mazeToggle) {
		mazeToggle.addEventListener("click", () => {
			const mazeOnly = document.body.classList.toggle("maze-only");
			mazeToggle.setAttribute("aria-pressed", String(mazeOnly));
			mazeToggle.textContent = mazeOnly ? "Show site" : "View maze";
		});
	}

	const mazeDisable = document.getElementById("maze-disable");
	if (mazeDisable) {
		mazeDisable.addEventListener("click", () => {
			mazeEnabled = !mazeEnabled;
			document.body.classList.toggle("maze-disabled", !mazeEnabled);
			mazeDisable.setAttribute("aria-pressed", String(!mazeEnabled));
			mazeDisable.textContent = mazeEnabled ? "Disable maze" : "Enable maze";
			if (mazeEnabled) draw();
			else ctx.clearRect(0, 0, viewportWidth, viewportHeight);
		});
	}

	resize();
	window.requestAnimationFrame(animationFrame);
})();
