generateRange = function (min, max) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
};

const snowflakeCount = Math.floor(window.innerWidth / 50);
// Lower is faster
const verticalSpeed = 8;
const horizontalAdjustmentRange = 2;

deleteSnowflakes = function () {
  const snowflakes = document.querySelectorAll("svg.fa-snowflake");
  for (const snowflake of snowflakes) {
    snowflake.remove();
  }
};

createSnowflakes = function () {
  for (i = 0; i < snowflakeCount; i++) {
    generateSnowflake(true);
  }
};

document.addEventListener("DOMContentLoaded", () => {
  createSnowflakes();
  self.requestAnimationFrame(animate);
});

self.addEventListener("blur", deleteSnowflakes);
self.addEventListener("focus", createSnowflakes);

function generateInitialPosition(pageLoad = false) {
  let initialVerticalOffset;
  if (pageLoad) {
    initialVerticalOffset = Math.floor(Math.random() * window.innerHeight);
  } else {
    initialVerticalOffset = Math.floor(
      Math.random() * window.innerHeight / 6,
    );
  }
  const initialHorizontalOffset = Math.floor(
    Math.random() * window.innerWidth,
  );
  return {
    top: initialVerticalOffset + "px",
    left: initialHorizontalOffset + "px",
  };
}

function generateSnowflake(pageLoad = false) {
  const snowflake = document.createElement("i");
  snowflake.classList.add("fa-snowflake");
  snowflake.classList.add("fa-thin");
  snowflake.classList.add("fa-sharp");
  snowflake.style.height = generateRange(10, 30) + "px";
  snowflake.style.width = generateRange(10, 30) + "px";
  snowflake.style.position = "absolute";
  snowflake.style.color = "white";
  snowflake.dataset.falling = "false";

  snowflake.style.top = generateInitialPosition(pageLoad).top;
  snowflake.style.left = generateInitialPosition(pageLoad).left;
  document.body.appendChild(snowflake);
}

animate = function (timestamp) {
  const flakes = document.querySelectorAll(
    'svg.fa-snowflake[data-falling="false"]',
  );
  for (const flake of flakes) {
    if (flake.dataset.startTime == undefined) {
      flake.dataset.startTime = timestamp;
    }

    if (flake.dataset.starty == undefined) {
      flake.dataset.starty = parseInt(flake.style.top, 10);
    }
    // Vertical movement
    flake.style.top = parseInt(flake.dataset.starty, 10) +
      Math.floor((timestamp - flake.dataset.startTime) / verticalSpeed) +
      "px";
    if (parseInt(flake.style.top, 10) > window.innerHeight) {
      flake.remove();
      generateSnowflake();
    }
    // Horizontal shake
    if (Math.floor(timestamp) % 20 == 0) {
      const horizontalAdjustment = Math.floor(
        Math.random() * horizontalAdjustmentRange,
      );
      const flip = Math.floor(Math.random() * 2);
      if (flip == 0) {
        flake.style.left = parseInt(flake.style.left) + horizontalAdjustment +
          "px";
      } else {
        flake.style.left = parseInt(flake.style.left) - horizontalAdjustment +
          "px";
      }
    }
  }
  self.requestAnimationFrame(animate);
};
