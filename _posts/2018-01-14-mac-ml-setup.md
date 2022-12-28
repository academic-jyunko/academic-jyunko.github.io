---
layout: post
title: "Mac Machine Learning Research & Dev Setup"
excerpt: "A summary of setting up a clean mac for pythonic machine learning research and development."
date: 2018-01-14
disqus_identifier: 20180114
mathjax: true
author: heytham fayek
---

**Update #1:** Post updated to reflect macOS' move to zsh and the move to Python3 (09/06/2020). 

Every year I do a clean install of the latest macOS, to mostly get rid of the things I accumulated throughout the year that I no longer use and force myself to do some housekeeping. 
Whether after a clean install or just getting my hands on a new Mac, I find myself going through the same process every time.
In this post, I document this workflow for my future self and hopefully for the benefit of others. 

# Before

Back in the day, backing up all of my data just before a clean install was a painful process but thanks to iCloud, which now takes care of backing up my bookmarks, passwords, etc., it is now a straightforward process.
All I have to do is copy my files in `~/$USER` to an external hard drive. 
P.S. I do this in addition to the Time Machine backup just in case!

# Format

Prepare a bootable macOS installer, erase the main drive, and install the OS.
A nice guide for this can be found [here](https://www.macrumors.com/how-to/perform-clean-install-macos-catalina/).

# Settings & Software

First, copy your files back and change the trackpad, keyboard, user, etc. settings back to your liking.
Don't forget to turn off font smoothing. 
I also have a bunch of dot files that contain paths, aliases, etc. to make things easier. 
I keep these files in this Github [repository](https://github.com/haythamfayek/dotfiles), some of these files might be useful.

Also, install your favorite editor, TeX/iWork/Office, and other software you regularly rely on, as well as, themes and colors.

The remainder of the workflow will be run from the terminal.

# Command Line Tools

Second, we'll install the command line tools:

{% highlight shell %}
xcode-select --install 
{% endhighlight %}

Don't forget to accept the license agreement after installation before moving on if you're installing Xcode.

# Package Manager

Install a package manager, [Homebrew](https://brew.sh) (or [Macports](https://www.macports.org)):

{% highlight shell %}
/usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"
{% endhighlight %}

# Some Standard Packages

A version control system - [Git](https://git-scm.com/) (or preferred alternative): 

{% highlight shell %}
brew install git
git config --global user.name "Your Name Here"
git config --global user.email "your_email@youremail.com"
git config --global credential.helper osxkeychain
{% endhighlight %}

A minimal [.gitconfig](https://github.com/haythamfayek/dotfiles/.gitconfig) for Git settings.

A C compiler - [GCC](https://gcc.gnu.org); this is also a dependency for later packages:

{% highlight shell %}
brew install gcc
{% endhighlight %}

Python3:

{% highlight shell %}
brew install python
{% endhighlight %}

The world's best text editor :) - [Vim](http://www.vim.org): 

{% highlight shell %}
brew install vim
{% endhighlight %}

A minimal [.vimrc](https://github.com/haythamfayek/dotfiles/.vimrc) for some sensible vim settings.

It's useful to have Ruby and Gem installed:

{% highlight shell %}
brew install ruby
{% endhighlight %}

This website is built with [jekyll](https://jekyllrb.com):

{% highlight shell %}
gem install --user-install bundler jekyll
{% endhighlight %}

# Scientific Stack

I used to follow a somewhat involved process. 
Now, installing the usual suspects [NumPy](http://www.numpy.org), [SciPy](https://www.scipy.org), [MatPlotLib](https://matplotlib.org), [Jupyter](http://jupyter.org), and [Virtualenv](https://virtualenv.pypa.io/en/stable/) is as simple as:

{% highlight shell %}
python3 -m pip install numpy
python3 -m pip install scipy
python3 -m pip install matplotlib
python3 -m pip install jupyter
python3 -m pip install virtualenv
{% endhighlight %}

Other packages should probably be installed within a virtual environment.

(While you're at it, do consider donating :) to the open-source community.)

Finally, the machine learning toys - [scikit-learn](http://scikit-learn.org) and [PyTorch](http://pytorch.org): 

{% highlight shell %}
python3 -m pip install scikit-learn
python3 -m pip install torch
{% endhighlight %}

# End

That's all folks!
I find this to be a minimal setup to get up and running.
Other tools can be installed later when needed and probably within the project scope. 

<br>

**Citation:**
{% highlight tex %}
@misc{fayek2018,
  title   = "Mac Machine Learning Research & Dev Setup",
  author  = "Haytham M. Fayek",
  year    = "2018",
  url     = "https://haythamfayek.com/2018/01/14/mac-ml-setup.html"
}
{% endhighlight %}
