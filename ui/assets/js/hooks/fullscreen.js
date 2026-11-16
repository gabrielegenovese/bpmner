const Fullscreen = {
  mounted() {
    this.onClick = event => {
      const trigger = event.target.closest("[data-fullscreen-toggle]")
      if (!trigger || !this.el.contains(trigger)) return

      if (document.fullscreenElement) {
        document.exitFullscreen()
      } else if (this.el.requestFullscreen) {
        this.el.requestFullscreen()
      }
    }

    this.onFullscreenChange = () => this.sync()

    this.el.addEventListener("click", this.onClick)
    document.addEventListener("fullscreenchange", this.onFullscreenChange)

    this.sync()
  },

  updated() {
    this.sync()
  },

  destroyed() {
    this.el.removeEventListener("click", this.onClick)
    document.removeEventListener("fullscreenchange", this.onFullscreenChange)
  },

  sync() {
    const active = document.fullscreenElement === this.el
    this.el.classList.toggle("is-fullscreen", active)

    this.el.querySelectorAll("[data-fullscreen-toggle]").forEach(button => {
      button.setAttribute("aria-pressed", active.toString())
      button.title = active ? "Exit full screen" : "Enter full screen"
    })

    requestAnimationFrame(() => window.dispatchEvent(new Event("resize")))
  },
}

export default Fullscreen
