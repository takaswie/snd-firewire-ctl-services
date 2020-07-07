========================
snd-firewire-ctl-services
========================

2020/06/03
Takashi Sakamoto

Introduction
============

This project is a sub project in Advanced Linux Sound Architecture a.k.a ALSA,
to produce userspace service daemon for Audio and Music units on IEEE 1394 bus,
supported by drivers in ALSA firewire stack.

License
=======

GNU General Public License Version 3

Design note
===========

Control model
-------------

.. image:: control-model.png
   :alt: control model

Monitor model
-------------

.. image:: monitor-model.png
   :alt: monitor model

Listener model (with help of drivers in ALSA firewire stack)
-------------------------------------------------------------------

.. image:: listener-model-a.png
   :alt: listener-a-model

Listener model (without any help of drivers in ALSA firewire stack)
-------------------------------------------------------------------

.. image:: listener-model-b.png
   :alt: listener-b-model

Multi threading
---------------

.. image:: overview.png
   :alt: overview